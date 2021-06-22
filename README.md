# EnvGrep

Linux CLI utility to find environment variable keys or values in all processes
that you have permissions to read.

## Example

Let's imagine that we have a long-running server process that have a few
environment variables that are used for configuration: `SERVER_VERSION`,
`SERVER_BIND_PORT`, and `SERVER_ID`. We can simulate this using a few `tail -f
/dev/null` processes.

```
$ env SERVER_VERSION=1.0.1 SERVER_ID=1 SERVER_BIND_PORT=:8080 tail -f /dev/null &
[1] 1969
$ env SERVER_VERSION=1.0.1 SERVER_ID=2 SERVER_BIND_PORT=:9090 tail -f /dev/null &
[2] 1970
$ env SERVER_VERSION=1.0.1 SERVER_ID=3 SERVER_BIND_PORT=:8181 tail -f /dev/null &
[3] 1971
```

Now that these processes are running and have their variables set, we can search
both keys and values using `envgrep`.

```
# To find all configuration variables for all servers
$ envgrep SERVER
/proc/1969/environ (tail -f /dev/null):
SERVER_VERSION = "1.0.1"
SERVER_ID = "1"
SERVER_BIND_PORT = ":8080"

/proc/1970/environ (tail -f /dev/null):
SERVER_VERSION = "1.0.1"
SERVER_ID = "2"
SERVER_BIND_PORT = ":9090"

/proc/1971/environ (tail -f /dev/null):
SERVER_VERSION = "1.0.1"
SERVER_ID = "3"
SERVER_BIND_PORT = ":8181"

# To find only the IDs for the server processes
$ envgrep SERVER_ID
/proc/1969/environ (tail -f /dev/null):
SERVER_ID = "1"

/proc/1970/environ (tail -f /dev/null):
SERVER_ID = "2"

/proc/1971/environ (tail -f /dev/null):
SERVER_ID = "3"

# To find the specific process that is running on port 8181
$ envgrep 8181
/proc/1971/environ (tail -f /dev/null):
SERVER_BIND_PORT = ":8181"
```

## Installing

`envgrep` is currently not packaged with any distro's package manager, so you
must rely on `cargo` to compile and install from source.

```
$ cargo install envgrep
```

## Command-line options

```
envgrep 0.1.0
Search through the environment variables of all running processes on the system and report on all variables that match
the specified pattern

USAGE:
    envgrep [FLAGS] <PATTERN>

FLAGS:
    -i, --case-insensitive    Perform case-insensitive matching with the specified regex
    -h, --help                Prints help information
    -V, --version             Prints version information
    -v, --verbose             Print all error messages as they occur instead of hiding them

ARGS:
    <PATTERN>    Regex pattern to use to search for environment variables. Matches on both parts of the `KEY=value`
                 string (independently), so parts of the environment variable name, value, or both can be used here``
```

## Limitations

Envgrep can search through procfs for environment variables, but applications
can also modify their own environments. Many executables do not rewrite changes
to their environment back into procfs, so if your executable modifies its own
environment it may not show up in the output of envgrep.

The tool currently relies on procfs, so it only works on operating systems that
support procfs (no macOS or Windows support).
