# Downloader Roadmap

Here are some fake "tickets" for finishing this challenge:

0. [ ] Download a file in sequential chunks (requires range request)
1. [ ] Download a file in parallel chunks (requires chunk hash map)
2. [ ] Download file faster by optimizing chunk size / file size ratio
3. [ ] Resume downloading a file if program crashes midway through download (requires serializing hash map state)
4. [ ] Download file when server does not respond to HEAD request (requires discovering file size via range requests)
