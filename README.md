"# Zimple-Server" 

Local/Private HTTP server framework built in rust.

Ideally the file format should be:
======================
+ Folder
| - Server Executable
| + server_content
| | ... Files (.html)
======================

So that:
server.exe calls for ./server_content/file.html

While Src / Dev:
./server/server_content/file.html
