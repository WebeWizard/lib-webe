/*
  Accounts - used to authenticate a single user
  Users - contains user specific information like name, email, etc.
  Session - A result of logging in and selecting a user.
   - Contains a token for accessing user specific data
   - Contains an expiration date for timing out and forcing re-auth

  TODO - for every db table, have two users, neither of which can edit table structure/permission
   - one that can insert/update/select/delete
   - one that can only read
*/

pub mod models;