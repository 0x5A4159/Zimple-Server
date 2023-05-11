# Zimple-Server

**Local/Private HTTP server framework built in rust.**
Created as a way to learn the HTTP structure.

### Usage:
In its current iteration, Zimple serves based on get requests and loads the file requested.
If there isn't a file, or the request contains relational searching characters such as '..' or '~' it will return the 404.html page by default.

### File Structuring:
The file structure for the server should be set up like:
+ server folder
  - Executable
  - config.cfg
  + server_content
    - servers payloads (.html, .css, .js, .etc)

During dev it would be:
- src
+ server folder
  - config.cfg
  + server_content
    - server payloads

+ To-do:
  - [] HTTPS
  - [x] GET
  - [] POST
  - [] PUT
  - [] DELETE
  - [] PATCH
