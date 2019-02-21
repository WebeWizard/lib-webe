Using the term 'Customer' to refer to the person using the system.

## Authentication - lib-webe::auth

### Account Creation:
A similar structure to Netflix accounts.  Each account can have multiple Users. Users are essentially just logical separation with no security between Users on the same Account.

*Accessing account settings should always ask for account credentials, regardless of existing sessions.*

#### Process
 - Customer creates an Account by supplying unique email and a password.
 - The Account gets created in an un-verified state and the system emails the customer a verification link with expiration.

### Account Verification:
The verification processes helps protect the system from holding on to accounts that get created but never used.  The system should purge unverified accounts periodically.

#### Process 
The user clicks the link
- If link has expired (or no account matches the code)
  - Let the customer know and ask them to re-register the account. 
- Else,
  - Ask the customer for a default User name.
  - Ask the customer to provide their account password.
  - If password is valid, 
    - sets the 'verified' flag on the account to `true`.  
    - create a new default User
    - create a new Session for the User.

### User Creation:
Users contain information that identifies the customer using the system.  User records contain details that help customize the experience of using the system.

#### Process

Customer must already have a valid session.
- Ask the customer for a new User name.
- Ask the customer to enter account password.
- If password is valid,
  - create a new user

### Session Creation (Logging In):
A session represents the result of providing account credentials  
An existing session can be updated later when the customer selects a user.

#### Process

Customer provides account email and password.
- If email matches an actual account and password is valid,
  - If account is verified
    - return a new session with account_id for sure, and user_id if it's provided.
  - Else,
    - ask the customer to check email for verifaction link, or ask if we should resend
- Else,
  - return a login error.

## HTTP Server

Assumes that everyone in the world is friendly and no one would ever try to abuse it.

