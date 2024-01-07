use std::boxed::Box;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::io::BufReader;

use super::Request;
use super::Responder;
use super::Response;
use super::Status;
use super::Validation;
use super::ValidationResult;
use crate::constants::{DEFAULT_MIME_TYPES, MIME_OCTET_STREAM};

pub enum MimeTypeList {
    Default,
    Custom(Vec<(String, String)>),
}

pub struct FileResponder {
    mount_point: PathBuf,
    path_param: String, // specifies the route parameter that provides file path relative to mount point
    use_index: bool,
    mime_types: MimeTypeList,
}

#[derive(Debug)]
pub enum FileResponderError {
    BadPath,
}

impl FileResponder {
    pub fn new(
        mount_point: String,
        path_param: String,
    ) -> Result<FileResponder, FileResponderError> {
        let mount_point = PathBuf::from(mount_point);
        match mount_point.canonicalize() {
            Ok(abs_path) => Ok(FileResponder {
                mount_point: abs_path,
                path_param: path_param,
                use_index: true,
                mime_types: MimeTypeList::Default,
            }),
            Err(_error) => return Err(FileResponderError::BadPath),
        }
    }

    // if this responder has a custom list of mime types, use it. otherwise use crate const default list
    pub fn find_mime_type(&self, file_path: &PathBuf) -> &str {
        match file_path.extension() {
            Some(extension) => match &self.mime_types {
                MimeTypeList::Default => {
                    match DEFAULT_MIME_TYPES
                        .iter()
                        .find(|mime_type| mime_type.0 == extension)
                    {
                        Some(result) => return result.1,
                        None => MIME_OCTET_STREAM,
                    }
                }
                MimeTypeList::Custom(list) => match list
                    .iter()
                    .find(|mime_type| mime_type.0.as_str() == extension)
                {
                    Some(result) => return result.1.as_str(),
                    None => MIME_OCTET_STREAM,
                },
            },
            None => MIME_OCTET_STREAM,
        }
    }

    fn validate_get_path(&self, file_path: PathBuf) -> ValidationResult {
        // make sure that the full path is a child of the mount point
        // this also makes sure the file or directory actually exists
        match file_path.canonicalize() {
            Ok(abs_file_path) => {
                // at the moment we only return files. no directory
                if abs_file_path.starts_with(&self.mount_point) {
                    if abs_file_path.is_file() {
                        return Ok(Some(Box::new(abs_file_path)));
                    } else if self.use_index && abs_file_path.is_dir() {
                        // check for index.html or index.html
                        if abs_file_path.join("index.html").is_file() {
                            return Ok(Some(Box::new(abs_file_path.join("index.html"))));
                        } else if abs_file_path.join("index.htm").is_file() {
                            return Ok(Some(Box::new(abs_file_path.join("index.htm"))));
                        }
                    }
                    return Err(Status::from_standard_code(404));
                } else {
                    return Err(Status::from_standard_code(404)); // not in mounted directory or not a file
                }
            }
            Err(_error) => return Err(Status::from_standard_code(404)), // not found or failed to canonicalize
        }
    }

    // just make sure the full path is within the mount point
    fn validate_put_path(&self, file_path: PathBuf) -> ValidationResult {
        match file_path.canonicalize() {
            Ok(abs_file_path) => {
                if abs_file_path.starts_with(&self.mount_point) {
                    if abs_file_path.is_dir() {
                        return Err(Status::from_standard_code(404)); // path is a dir, can't replace dirs
                    }
                    if abs_file_path.is_symlink() {
                        return Err(Status::from_standard_code(404)); // can't replace symlinks
                    }
                    return Ok(Some(Box::new(abs_file_path)));
                } else {
                    return Err(Status::from_standard_code(404)); // not in mounted directory or not a file
                }
            }
            Err(_error) => return Err(Status::from_standard_code(404)), // not found or failed to canonicalize
        }
    }

    // returns a response with a file reader from the filesystem
    fn respond_to_get(&self, request: &Request, path_box: Box<PathBuf>) -> Result<Response, u16> {
        match path_box.metadata() {
            Ok(meta) => {
                let size = meta.len();
                match File::open(path_box.as_ref()) {
                    Ok(file) => {
                        // build the response
                        let mut headers = HashMap::<String, String>::new();
                        headers.insert("Content-Length".to_owned(), size.to_string());
                        headers.insert(
                            "Content-Type".to_owned(),
                            self.find_mime_type(&path_box).to_string(),
                        );
                        let mut response = Response::new(200);
                        response.headers = headers;
                        response.message_body =
                            Some(Box::pin(BufReader::new(tokio::fs::File::from_std(file))));
                        return Ok(response);
                    }
                    Err(_error) => return Err(500),
                }
            }
            Err(_error) => return Err(500),
        }
    }

    fn respond_to_put(&self, request: &Request, path: Box<PathBuf>) -> Result<Response, u16> {
      return Err(501);
    }
}

#[async_trait]
impl Responder for FileResponder {
    // tests if the provided path exists
    fn validate(
        &self,
        request: &Request,
        params: &Vec<(String, String)>,
        _validation: Validation,
    ) -> ValidationResult {
        match params
            .into_iter()
            .find(|(key, _value)| *key == self.path_param)
        {
            Some((_key, path_string)) => {
                // build the full path
                let mut file_path = PathBuf::new();
                file_path.push(&self.mount_point);
                file_path.push(PathBuf::from(path_string));

                match request.method.as_str() {
                    "GET" => return self.validate_get_path(file_path),
                    "PUT" => return self.validate_put_path(file_path),
                    _ => return Err(Status::from_standard_code(405)), // method not allowed
                }
            }
            None => return Err(Status::from_standard_code(500)), // no path provided
        }
    }

    async fn build_response(
        &self,
        request: &mut Request,
        _params: &Vec<(String, String)>,
        validation: Validation,
    ) -> Result<Response, u16> {
        // use the path contained in the validation
        match validation {
            Some(any_box) => {
                match any_box.downcast::<PathBuf>() {
                    Ok(path_box) => {
                        match request.method.as_str() {
                            "GET" => return self.respond_to_get(request, path_box),
                            "PUT" => return self.respond_to_put(request, path_box),
                            _ => return Err(405), // method not allowed
                        }
                    }
                    Err(_error) => return Err(500),
                }
            }
            None => return Err(500),
        }
    }
}
