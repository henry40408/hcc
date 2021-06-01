# hcc

[![Build Status](https://ci.h08.io/api/badges/henry40408/hcc/status.svg)](https://ci.h08.io/henry40408/hcc)

**H**TTPS **C**ertificate **C**heck

## Installation

Running as Docker container:

```bash
$ make build-docker-image
$ docker run -it -p 9292:9292 henry40408/hcc /hcc-server -b 0.0.0.0:9292docker run -it -p 9292:9292 henry40408/hcc /hcc-server -b 0.0.0.0:9292
```

Or run directly:

```bash
$ cargo run --bin hcc-server
```

## Usage

```bash
$ curl :9292/sha512.badssl.com
{"state":"OK","checked_at":"2021-06-01T07:45:24+00:00","days":304,"domain_name":"sha512.badssl.com","expired_at":"2022-04-01T12:00:00+00:00","elapsed":364}

$ curl :9292/expired.badssl.com
{"state":"EXPIPRED","checked_at":"2021-06-01T07:45:24+00:00","days":0,"domain_name":"expired.badssl.com","expired_at":"1970-01-01T00:00:00+00:00","elapsed":0}

$ curl :9292/sha512.badssl.com,expired.badssl.com
[{"state":"OK","checked_at":"2021-06-01T07:45:24+00:00","days":304,"domain_name":"sha512.badssl.com","expired_at":"2022-04-01T12:00:00+00:00","elapsed":172},{"state":"EXPIPRED","checked_at":"2021-06-01T07:45:24+00:00","days":0,"domain_name":"expired.badssl.com","expired_at":"1970-01-01T00:00:00+00:00","elapsed":0}]
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)
