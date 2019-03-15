use std::boxed::Box;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use super::Request;
use super::Responder;
use super::Response;

#[derive(Clone)]
pub struct FileResponder {
    mount_point: PathBuf,
    path_param: String,// specifies the route parameter that provides file path relative to mount point
}

pub enum FileResponderError {
    BadPath
}

impl FileResponder {
    pub fn new (mount_point: String, path_param: String) -> Result<FileResponder,FileResponderError> {
        let mount_point = PathBuf::from(mount_point);
        match mount_point.canonicalize() {
            Ok(abs_path) => {
                Ok(FileResponder { mount_point: abs_path, path_param: path_param })
            },
            Err(_error) => return Err(FileResponderError::BadPath)
        }
    }
}

impl Responder for FileResponder {
    // tests if the provided path exists
    fn validate(&self, request: &Request, params: &HashMap<String,String>) -> bool {
        println!("validating");
        match params.get(&self.path_param) {
            Some(path_string) => {
                // build the full path
                let mut file_path = PathBuf::new();
                file_path.push(&self.mount_point);
                file_path.push(PathBuf::from(path_string));

                println!("{:?}",file_path);
                // make sure that the full path is a child of the mount point
                // this also makes sure the file or directory actually exists
                match file_path.canonicalize() {
                    Ok(abs_file_path) => {
                        println!("{:?}",abs_file_path);
                        // at the moment we only return files. no directory
                        return abs_file_path.starts_with(&self.mount_point) && abs_file_path.is_file();
                    },
                    Err(_error) => return false
                }
            },
            None => return false // no path provided
        }
    }

    // TODO: Currently only using identity encoding
    fn build_response(&self, request: &Request, params: &HashMap<String,String>) -> Response {
        // get the size of the file
        match params.get(&self.path_param) {
            Some(path_string) => {
                // build the full path and open the file
                let mut file_path = PathBuf::new();
                file_path.push(&self.mount_point);
                file_path.push(PathBuf::from(path_string));
                match file_path.canonicalize() {
                    Ok(abs_file_path) => {
                        match abs_file_path.metadata() {
                            Ok(meta) => {
                                let size = meta.len();
                                match File::open(abs_file_path) {
                                    Ok(file) => {
                                        // build the response
                                        let mut headers = HashMap::<String, String>::new();
                                        headers.insert("Content-Length".to_owned(), size.to_string());
                                        let mut response = Response::new(200);
                                        response.headers = headers;
                                        response.message_body = Some(Box::new(BufReader::new(file)));
                                        return response;
                                    },
                                    Err(_error) => return Response::new(500)
                                }
                            },
                            Err(_error) => return Response::new(500)
                        }
                    },
                    Err(_error) => return Response::new(500)
                }
            },
            None => return Response::new(500)
        }
    }
}
