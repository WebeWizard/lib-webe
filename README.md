## HTTP Server - lib-webe::http

#### Server
The server accepts incomming tcp connections.  From the connection's stream it will read and form requests.
For the request the server will find the best matching route.  The route will use its provided responder to generate a Response for the request.  The server will then send the response back over the tcp stream.

#### Route
Has 2 parts:  a tuple of (Method,RouteURI) and a Responder.

#### Responder
Responder is a trait with two functions: `validate` and `build_response`.  
`validate` is intended to be a fast way to verify if the request is worth responding to.  
It returns a Result<u16,u16> where the u16 represents a status code to send to the next operation. Ok(u16) will tell the server to call the responder's `build_response` function and hint to the function what http status is appropriate. Err(u16) will tell the server to call the server's static_message responder which always builds a response with the provided u16 http status code.


## Authentication - lib-webe::auth

Using the term 'Customer' to refer to the person using the system.

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
    - begin user creation process

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

