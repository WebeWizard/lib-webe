use std::env;
use webe_args::args::{ArgError, ArgOpts, Args};

// NOTE:  we have to circumvent the built-in cargo test harness
// in order to pass command line arguments into the test
// NOTE:  Run this cargo test with the following command:
// cargo test -p webe_args --test cli -- --nocapture -- -- --with_value this_is_value --as_flag ignore_this -s --bad_val WebeIsBad --good_val "webe is great"
fn main() {
  println!("Command Line Arguments:");
  println!("{:?}", std::env::args().collect::<Vec<String>>());
  println!();
  // test arguments with_value
  println!("Running Test:  with_value()...");
  with_value();
  println!("Ok");
  println!();

  // test arguments as flags
  println!("Running Test:  as_flag()...");
  as_flag();
  println!("Ok");

  // test getting arguments with short name
  println!("Running Test:  get_short()...");
  get_short();
  println!("Ok");

  // test making sure a value passes validatoin
  println!("Running Test:  with_validation()...");
  with_validation();
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
      Some(value) => panic!("Flags should not have any values"),
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
      Some(value) => {}
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
      Some(value) => panic!("Flags should not have any values"),
      None => {}
    },
    Err(err) => panic!(
      "Eror getting argument 'short_arg' *by the short name* from command line '-s': {:?}",
      err
    ),
  }
}

fn default_args() -> Args {
  let mut args = Args::new();
  args.add(
    "with_value".to_owned(),
    ArgOpts {
      short: None,
      description: Some("Test of argument with a required value".to_owned()),
      is_required: false,
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
