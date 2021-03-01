use std::panic::{self, AssertUnwindSafe};

use webe_args::args::{ArgError, ArgOpts, Args};

// NOTE:  we have to circumvent the built-in cargo test harness
// in order to pass command line arguments into the test
// NOTE:  Run this cargo test with the following command:
// cargo test -p webe_args --test cli -- --nocapture -- -- --with_value this_is_value --as_flag ignore_this -s --bad_val WebeIsBad --good_val "webe is great"
fn main() {
  // --- Prepare for Tests ---
  // verify that this test was launched with required command line input
  let expected_args = vec!("--nocapture", "--", "--", "--with_value", "this_is_value", "--as_flag", "ignore_this", "-s", "--bad_val", "WebeIsBad", "--good_val", "webe is great");
  let mut provided_args = std::env::args().collect::<Vec<String>>();
  provided_args.remove(0); // removes the first env_arg because it contains the target path
  if expected_args != provided_args {
    panic!("\r\nUnexpected command line text provided.  Please run this test using the following text:\r\n{}", "cargo test -p webe_args --test cli -- --nocapture -- -- --with_value this_is_value --as_flag ignore_this -s --bad_val WebeIsBad --good_val \"webe is great\"");
  }

  // --- Run Tests ---
  // test arguments with_value
  print!("Running Test:  with_value()...");
  with_value();
  println!("Ok");

  // test arguments as flags
  print!("Running Test:  as_flag()...");
  as_flag();
  println!("Ok");

  // test getting arguments with short name
  print!("Running Test:  get_short()...");
  get_short();
  println!("Ok");

  // test making sure a value passes validation
  print!("Running Test:  with_validation()...");
  with_validation();
  println!("Ok");

  // test finding missing required arguments
  print!("Running Test:  missing_required");
  missing_required();
  println!("Ok");

  // test that all required args are present and valid
  print!("Running Test:  parse_args()...");
  parse_args();
  println!("Ok");
}

fn with_value() {
  let args = default_args();
  match args.get("with_value") {
    Ok(value_opt) => match value_opt {
      Some(value) => assert_eq!(value, "this_is_value"),
      None => panic!("The argument 'with_value' should have provided a value"),
    },
    Err(err) => panic!(
      "Error getting argument from command line 'with_value': {:?}",
      err
    ),
  }
}

fn as_flag() {
  let args = default_args();
  match args.get("as_flag") {
    Ok(value_opt) => match value_opt {
      Some(_value) => panic!("Flags should not have any values"),
      None => {}
    },
    Err(err) => panic!(
      "Eror getting argument from command line 'as_flag': {:?}",
      err
    ),
  }
}

fn with_validation() {
  let args = default_args();
  match args.get("bad_val") {
    Ok(_) => panic!("The value for argument 'bad_val' should not have passed validation"),
    Err(err) => match err {
      ArgError::InvalidValue => {}
      _ => panic!(
        "Error getting argument from command line 'bad_val': {:?}",
        err
      ),
    },
  }
  match args.get("good_val") {
    Ok(value_opt) => match value_opt {
      Some(_value) => {}
      None => panic!("The argument 'good_val' should have provided a value"),
    },
    Err(err) => panic!(
      "Eror getting argument 'good_val' from command line: {:?}",
      err
    ),
  }
}

fn get_short() {
  let args = default_args();
  // will get from command line as '-s'
  match args.get("short_arg") {
    Ok(value_opt) => match value_opt {
      Some(_value) => panic!("Flags should not have any values"),
      None => {}
    },
    Err(err) => panic!(
      "Eror getting argument 'short_arg' *by the short name* from command line '-s': {:?}",
      err
    ),
  }
}

fn missing_required() {
  let args = default_args();
  match args.get("missing_required") {
    Ok(_value) => panic!("The argument 'missing_required' should be missing from cli arguments"),
    Err(err) => match err {
      ArgError::RequiredNotFound => {},
      _ => panic!("Error getting argument from the command line 'missing_required'")
    }
  }
}

fn parse_args() {
  let bad_args = AssertUnwindSafe(default_args());
  // catch the panic from the library (like the #[should_panic] attribute in cargo's ootb test harness)
  match panic::catch_unwind(|| {
    // silence the library's panic output
    panic::set_hook(Box::new(|_| {}));
    bad_args.parse_args();
    let _ = panic::take_hook(); // restore panic output
  }) {
    Ok(_result) => panic!("Parsing arguments should have failed"),
    Err(_cause) => {} // parse_args() panicked as expected
  }

  let mut good_args = Args::new();
  good_args.add(
    "with_value".to_owned(),
    ArgOpts {
      short: None,
      description: Some("Test of argument with a required value".to_owned()),
      is_required: true,
      is_flag: false,
      validation: None,
    },
  );
  good_args.add(
    "short_arg".to_owned(),
    ArgOpts {
      short: Some("s".to_owned()),
      description: Some("Test of argument with a value that requires simple validation".to_owned()),
      is_required: false,
      is_flag: true,
      validation: None,
    },
  );
  good_args.add(
    "as_flag".to_owned(),
    ArgOpts {
      short: Some("f".to_owned()),
      description: Some("Test of argument without value (flag)".to_owned()),
      is_required: false,
      is_flag: true,
      validation: None,
    },
  );
  let good_args = AssertUnwindSafe(good_args);
  match panic::catch_unwind(|| good_args.parse_args()) {
    Ok(_result) => {} // did not panic as expected.
    Err(_cause) => panic!("Parsing arguments should NOT have paniced"),
  }

}

fn default_args() -> Args {
  let mut args = Args::new();
  args.add(
    "missing_required".to_owned(),
    ArgOpts {
      short: None,
      description: Some("This argument is present, but should not be found in args".to_owned()),
      is_required: true,
      is_flag: false,
      validation: None,
    },
  );
  args.add(
    "with_value".to_owned(),
    ArgOpts {
      short: None,
      description: Some("Test of argument with a required value".to_owned()),
      is_required: true,
      is_flag: false,
      validation: None,
    },
  );
  args.add(
    "short_arg".to_owned(),
    ArgOpts {
      short: Some("s".to_owned()),
      description: Some("Test of argument with a value that requires simple validation".to_owned()),
      is_required: false,
      is_flag: true,
      validation: None,
    },
  );
  args.add(
    "as_flag".to_owned(),
    ArgOpts {
      short: Some("f".to_owned()),
      description: Some("Test of argument without value (flag)".to_owned()),
      is_required: false,
      is_flag: true,
      validation: None,
    },
  );
  args.add(
    "bad_val".to_owned(),
    ArgOpts {
      short: None,
      description: Some("Test of argument with a value that should fail validation".to_owned()),
      is_required: false,
      is_flag: false,
      validation: Some(Box::new(|input| input == "webe is great")),
    },
  );
  args.add(
    "good_val".to_owned(),
    ArgOpts {
      short: None,
      description: Some("Test of argument with a value that should pass validation".to_owned()),
      is_required: false,
      is_flag: false,
      validation: Some(Box::new(|input| input == "webe is great")),
    },
  );
  return args;
}
