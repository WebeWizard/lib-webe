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
  path_param: String, // specifies the route parameter that provides file path relative to mount point
  use_index: bool,
}

#[derive(Debug)]
pub enum FileResponderError {
  BadPath,
}

impl FileResponder {
  pub fn new(
    mount_point: String,
    path_param: String,
    use_index: bool,
  ) -> Result<FileResponder, FileResponderError> {
    let mount_point = PathBuf::from(mount_point);
    match mount_point.canonicalize() {
      Ok(abs_path) => Ok(FileResponder {
        mount_point: abs_path,
        path_param: path_param,
        use_index: use_index,
      }),
      Err(_error) => return Err(FileResponderError::BadPath),
    }
  }
}

impl Responder for FileResponder {
  // tests if the provided path exists
  fn validate(&self, _request: &Request, params: &HashMap<String, String>) -> Result<u16, u16> {
    match params.get(&self.path_param) {
      Some(path_string) => {
        // build the full path
        let mut file_path = PathBuf::new();
        file_path.push(&self.mount_point);
        file_path.push(PathBuf::from(path_string));

        // make sure that the full path is a child of the mount point
        // this also makes sure the file or directory actually exists
        match file_path.canonicalize() {
          Ok(abs_file_path) => {
            // at the moment we only return files. no directory
            if abs_file_path.starts_with(&self.mount_point) {
              if abs_file_path.is_file() {
                return Ok(200);
              } else if self.use_index && abs_file_path.is_dir() {
                // check for index.html or index.html
                if abs_file_path.join("index.html").is_file()
                  || abs_file_path.join("index.htm").is_file()
                {
                  return Ok(200);
                }
              }
              return Err(404);
            } else {
              return Err(404); // not in mounted directory or not a file
            }
          }
          Err(_error) => return Err(404), // not found or failed to canonicalize
        }
      }
      None => return Err(500), // no path provided
    }
  }

  fn build_response(
    &self,
    _request: &mut Request,
    params: &HashMap<String, String>,
    validation_code: u16,
  ) -> Result<Response, u16> {
    // get the size of the file
    match params.get(&self.path_param) {
      Some(path_string) => {
        // build the full path and open the file
        let mut file_path = PathBuf::new();
        file_path.push(&self.mount_point);
        file_path.push(PathBuf::from(path_string));
        match file_path.canonicalize() {
          Ok(mut abs_file_path) => {
            if self.use_index && abs_file_path.is_dir() {
              // check for and use index.html or index.html
              if abs_file_path.join("index.html").is_file() {
                abs_file_path.push("index.html")
              } else if abs_file_path.join("index.htm").is_file() {
                abs_file_path.push("index.htm")
              }
            }
            match abs_file_path.metadata() {
              Ok(meta) => {
                let size = meta.len();
                match File::open(abs_file_path) {
                  Ok(file) => {
                    // build the response
                    let mut headers = HashMap::<String, String>::new();
                    headers.insert("Content-Length".to_owned(), size.to_string());
                    let mut response = Response::new(validation_code);
                    response.headers = headers;
                    response.message_body = Some(Box::new(BufReader::new(file)));
                    return Ok(response);
                  }
                  Err(_error) => return Err(500),
                }
              }
              Err(_error) => return Err(500),
            }
          }
          Err(_error) => return Err(500),
        }
      }
      None => return Err(500),
    }
  }
}
