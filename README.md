# One Library for One Wizard

This project aims to provide a set of tools for creating fast and stable Rust application servers suitable for a hobbyist developer.

## HTTP Server
Using HTTP 1.1 Spec as a reference but not a requirement.

Responders make up the basic request handler.  Responders can be nested to conveniently provide authentication, logging, or any repeated custom behavior on each request.

Responders are mapped to a Route.  The servers parses the URL of each request and chooses the best matching route.
Ex. Consider a server with 2 endpoints:  
`/post`,  
`/post/<post_num>`,  

A request to `/post/123` would match route `/post/\<post_num\>` because it contains the correct number of url parts. The parameter <post_num> can then read from the responder.  
A request to `/post/123/edit` would also match `/post/<post_num>`. And the value `123/edit` would be passed to the Responder.  We could configure the responder to parse this, or we could add an additional responder at route `/post/<post_num>/<action>`.

## Authentication
 - Account management (Basic Account CRUD operations)
 - BCrypt hashed passwords
 - Token based Sessions
 - Includes prebuilt HTTP server Responders


## Unique ID Generation
Custom unique id generator based on Twitter's Snowflake.  Currently located at https://github.com/WebeWizard/WebeID