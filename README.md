# Potential Giggle

[![Build Status](https://ci.h08.io/api/badges/henry40408/potential-giggle/status.svg)](https://ci.h08.io/henry40408/potential-giggle)

Potential Giggle is a SSL checking server.

## Installation

Running as Docker container:

```bash
$ make build-docker-image
$ docker run -it -p 9292:9292 henry40408/potential-giggle /potential-giggle-server -b 0.0.0.0:9292docker run -it -p 9292:9292 henry40408/potential-giggle /potential-giggle-server -b 0.0.0.0:9292
```

Or run directly:

```bash
$ cargo run --bin potential-giggle-server
```

## Usage

```bash
$ curl 127.0.0.1:9292/sha512.badssl.com
{"ok":true,"checked_at":"2021-03-01T12:39:01+00:00","days":395,"domain_name":"sha512.badssl.com","expired":false,"expired_at":"2022-04-01T12:00:00+00:00"}

$ curl 127.0.0.1:9292/expired.badssl.com
{"ok":false,"checked_at":"2021-03-01T12:39:22+00:00","days":0,"domain_name":"expired.badssl.com","expired":true,"expired_at":"1970-01-01T00:00:00+00:00"}
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)
