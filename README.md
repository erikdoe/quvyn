# About Quvyn

![Rust](https://github.com/erikdoe/quvyn/workflows/Rust/badge.svg)

Quvyn is a minimal website commenting system. It doesn't have an admin UI, it doesn't support multiple sites, it 
doesn't support load-balancing, it doesn't even use a database. But it comes as a single binary with no dependencies, 
it stores comments in the filesystem where they can be processed with lots of Unix tools, and it uses modern 
technologies.


## tl;dr

Step 0: Install Rust toolchain (if needed)

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Step 1: Clone Quvyn repo

    git clone git@github.com:erikdoe/quvyn.git && cd quvyn

Step 2: Build and run Quvyn

    cargo run -- --repo ./var/demo

Step 3: Demo time

    open http://localhost:8080/app/demo.html

Now you should see a page with a working commenting system. Play around. If you like what you see, read on.


## Installation

The Quvyn backend is written in [Rust](https://www.rust-lang.org/). So, you need the Rust toolchain to build a binary 
on your system. Once you have the Rust toolchain you can build Quvyn with the Rust package manager:

    cargo build --release

Assuming the build succeeds, the binary can be found in `target/release/quvyn`. Copy this binary to a suitable place 
on your system and set it up as a daemon process, using [supervisord](http://supervisord.org/) or 
[daemontools](https://cr.yp.to/daemontools.html) or similar.


## Configuration

There is no configuration file and Quvyn does not read environment variables. All configuration is done via command-line
options:

`--repo PATH`

The comments are stored as files in a directory, referred to as the repository. The location of the repository can be 
specified with this option. Each file in the repository contains a single comment in JSON format. The filenames are
random UUIDs.

When Quvyn starts it reads all files from the repository directory and keeps them in memory. When a new comment is 
posted, the comment is saved to the filesystem immediately. This explains why Quvyn does not scale horizontally, ie. 
you should not run multiple instances behind a load balancer.

`--app PATH`

Quvyn ships with a frontend written in [Vue.js](https://vuejs.org/), found in the `vue` directory in the source
distribution. You can use the Quvyn backend to serve its own frontend. To do so, make the directory available to the
Quvyn backend process and use this option to set the path.

**Note:** for normal use you want your website to load `quvyn.js`. You do not need the demo app or the stylesheet; both can 
serve as inspiration for how to integrate Quvyn into your own site, though. If you deploy all files a demo app is 
available at [http://localhost:8080/app/demo.html](http://localhost:8080/app/demo.html)

`--bind address`

By default Quvyn binds to port 80 on localhost. You can change this with this option. Hostname and port are separated
by a colon, eg. _0.0.0.0:4567_.

`--origin (URL|*)`

Quvyn can run on a domain different from the domain of the website where the comments are displayed. In such a case you 
must tell Quvyn to set appropriate [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) headers using this 
option. Normally, you would specify the URL of the website that displays the comments.

`--notify EMAIL-ADDRESS`

If this option is set Quvyn sends an email to the specified email address every time a comment is posted. The email
contains the comment as JSON.

**Note:** Quvyn simply uses `sendmail` to send the emails. So, please make sure that this is installed and works.


## Importing comments 

Quvyn can import comments from CSV files. The file **must** have the following format:

* Fields must be separated by commas (`,`).
* Fields that contain commas or line breaks must be place in double quotes (`"`).
* Double quotes in the field text must be escaped using a double quote, ie. a double quote in the field text is 
  written as a double double quote (`""`).
* The first line of the file must contain a header with the field names.
* Fields are as follows:

field (in order) | content
-----------------|---------
timestamp        | String in ISO 8601 format (actually, in RFC 3339 format)
path             | String that specifies the path of the page to which the comment belongs
author_name      | Name of the author (can be empty)
author_email     | Email address of the author (can be empty)
text             | Comment text in markdown format 

Such a file can then be imported with the `--import` option. Note that you should specify the repo path, too.



