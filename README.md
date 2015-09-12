unpack-rs
=========

This is a simple program that takes archive files and calls another program to
extract them. It was written for two reasons:

1. Computers are better than me at quickly recalling how to invoke `tar`
2. Computers are better than me at remembering that some archives are created
   by inconsiderate people who put multiple files into the top level of the
   archive and thus leave a mess all over my homedir

`unpack-rs` reads a bunch of lines of the form `.tar.gz: tar xfz` from
`~/.config/unpack-rs/formats` (see sample file `formats` file in this
repository) to figure out what to invoke for each command line argument. Then
it uses that invocation from inside a new hidden subdirectory of the current
working directory to extract the given archive. Then, if only one thing
appeared in that directory, the thing is moved back into the initial working
directory, otherwise the temporary directory is turned permanent and renamed to
the base name of the archive.

The whole thing is a bit unpolished, and I guess there's no particularly good
reason that this isn't just a shellscript. Oh well, it was fun to write, let's
hope it saves someone some frustration despite its shortcomings.
