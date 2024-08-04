A cross-platform command line utility to watch file system changes.

## install

```bash
cargo binstall chokidar
```

## npm
[chokidar-napi](https://github.com/ahaoboy/chokidar-napi)
```bash
npm i @chokidar-napi/chokidar
```

## example
```bash
chokidar 'src/**/*.{ts,tsx,json}' -c='pnpm run build' -d 1000
```
