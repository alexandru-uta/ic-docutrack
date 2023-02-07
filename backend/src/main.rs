use ic_cdk::export::candid::CandidType;
use ic_cdk_macros::query;
use ic_cdk_macros::update;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct User {
    first_name: String,
    last_name: String,
}

#[derive(CandidType, Serialize, Deserialize)]
enum WhoamiResponse {
    #[serde(rename = "known_user")]
    KnownUser(User),
    #[serde(rename = "unknown_user")]
    UnknownUser,
}

/// File metadata.
#[derive(CandidType, Serialize, Deserialize, Clone)]
struct FileMetadata {
    file_id: u64,
    file_name: String,
    // created_on: u64,
    // last_modified: u64,
}

// A file is composed of its metadata and its content, which is a blob.
struct File {
    metadata: FileMetadata,
    contents: Option<Vec<u8>>,
}

thread_local! {
    /// User names.
    static USERS: RefCell<BTreeMap<ic_cdk::export::Principal, User>> = RefCell::new(BTreeMap::new());

    /// Counts the number of uploaded files.
    static FILE_COUNT: RefCell<u64> = RefCell::new(0);

    /// File data.
    static FILE_DATA: RefCell<BTreeMap<u64, File>> = RefCell::new(BTreeMap::new());

    /// Mapping between file alias and file metadata
    static FILE_ALIAS_INDEX: RefCell<BTreeMap<String, FileMetadata>> = RefCell::new(BTreeMap::new());
}

#[query]
fn hello_world() -> String {
    format!("Hello {}!", ic_cdk::api::caller())
}

#[update]
fn set_user(first_name: String, last_name: String) {
    USERS.with(|users| {
        users.borrow_mut().insert(
            ic_cdk::api::caller(),
            User {
                first_name,
                last_name,
            },
        )
    });
}

#[query]
fn who_am_i() -> WhoamiResponse {
    USERS.with(|users| {
        let users = users.borrow();
        match users.get(&ic_cdk::api::caller()) {
            None => WhoamiResponse::UnknownUser,
            Some(user) => WhoamiResponse::KnownUser(user.clone()),
        }
    })
}

#[query]
fn get_request_info(alias: String) -> FileMetadata {
    FILE_ALIAS_INDEX.with(|file_alias_idx| {
        let file_alias_idx = file_alias_idx.borrow();
        match file_alias_idx.get(&alias) {
            Some(file_metadata) => file_metadata.clone(),
            None => FileMetadata {
                file_id: 0,
                file_name: "non-existing file".to_string(),
            },
        }
    })
}

#[update]
fn upload_file(file_id: u64, file_content: Vec<u8>) {
    FILE_DATA.with(|file_data| {
        let mut file_data = file_data.borrow_mut();
        if let Some(file) = file_data.get_mut(&file_id) {
            file.contents = Some(file_content.clone())
        }
    });
}

/// Gets file alias randomly, to be filled in
fn get_file_alias() -> String {
    "random".to_string()
}

#[update]
fn create_file_request(request_name: String) -> String {
    let crnt_file = FILE_COUNT.with(|file_count| {
        let mut counter = file_count.borrow_mut();
        *counter += 1;
        *counter
    });
    let file_metadata = FileMetadata {
        file_id: crnt_file,
        file_name: request_name.clone(),
    };
    let file = File {
        metadata: file_metadata.clone(),
        contents: None,
    };

    FILE_DATA.with(|file_data| {
        let mut file_data = file_data.borrow_mut();
        file_data.insert(crnt_file, file);
    });

    let file_alias = get_file_alias();
    let return_alias = file_alias.clone();
    FILE_ALIAS_INDEX.with(|file_alias_idx| {
        let mut file_alias_idx = file_alias_idx.borrow_mut();
        file_alias_idx.insert(file_alias, file_metadata);
    });

    // return dummy string for now
    return_alias
}

#[derive(CandidType, Serialize, Deserialize)]
enum FileData {
    #[serde(rename = "empty_file")]
    EmptyFile,
    #[serde(rename = "non_empty_file")]
    NonEmptyFile(Vec<u8>),
}

#[query]
fn download_file(file_id: u64) -> FileData {
    FILE_DATA.with(|file_data| {
        let file_data = file_data.borrow();
        match file_data.get(&file_id) {
            None => FileData::EmptyFile,
            Some(file) => {
                FileData::NonEmptyFile((*file.contents.as_ref().unwrap().clone()).to_vec())
            }
        }
    })
}

fn main() {}
