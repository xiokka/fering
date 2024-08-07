# fering
Simple static webring generator written in Rust.

## Usage
Create a new webring.
```bash
fering new_webring $webring_name
```
Example
```bash
fering new_webring "My new webring"
```
This will generate some basic project files and directories.

projectname.txt => contains project name.

static/base.html => base HTML file that will be used as a template to generate all pages. It contains placeholders which the generator function will replace accordingly.

static/about.html => The $CONTENT of the homepage. This also contains the placeholder $NAVCLOUD, which the generator function will replace with links to each tag index page (each tag index page contains links to all users for that tag).

static/style.css => a rather empty CSS file.

users/ => contains users.

images/ => contains images.

Add new user.
```bash
fering add_user $username $URL
```
Example
```bash
fering add_user Xiokka https://xiokka.neocities.org/
```
You must use the full URL.

Once a new user is added, the corresponding subdirectory is created inside the users directory. Inside the new directory, you will find three files: "tags.txt", "description.html" and "url.txt".
tags.txt contains the tags for that user, where each tag is separated by a whitespace. For example:
```
linux programming command_line
```
assigns tags "linux", "programming" and "command_line" to that user.

description.html contains HTML which will be inserted into the profile of that user in the webring hub. It can be used as a simple user introduction/profile inside the webring hub.

url.txt contains the URL for the website of that user.

Generate site
```bash
fering generate
```
This will create the public/ directory, where the webring site has been generated.

public/users/index.html contains all users in the webring.

public/$TAG/index.html contains all users tagged with $TAG in their tags.txt file.

Each user has a next.html and previous.html file, which redirect to the next and previous users.


To be redirected to the user next to $USERNAME:

public/users/$USERNAME/next.html


To be redirected to the user previous to $USERNAME:

public/users/$USERNAME/previous.html

where $USERNAME is a specific member of the webring.


For example, if your webring is hosted on "https://xiokka.neocities.org/webring/", you would be redirected to the user next to Xiokka by visiting:

https://xiokka.neocities.org/webring/users/Xiokka/next.html
