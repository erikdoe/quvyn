#!/bin/bash

rm -Rf _site
bundle install  --path _vendor/bundle
bundle exec jekyll serve --watch -b ""
