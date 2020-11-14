# Potential Giggle

Potential Giggle is a SSL checking server.

## Installation

Running as Docker container:

```bash
$ docker build -t henry40408/potential-giggle .
$ docker run -it -p 9292:9292 henry40408/potential-giggle
```

Or run directly:

```bash
$ bundle
$ bundle exec rackup -o 0.0.0.0
```

## Usage

```bash
$ curl www.lvh.me:9292/example.com
{"ok":true,"days":10,"seconds":945198}

$ curl www.lvh.me:9292/expired.badssl.com
{"ok":false}
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)
