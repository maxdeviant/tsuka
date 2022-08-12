const os = require('os');
const { configureProxy } = require('axios-proxy-builder');
const { binary, makeInstaller, run: runBinary } = require('install-bin');
const cTable = require('console.table');

const { name, version, repository } = require('./package.json');

const SUPPORTED_PLATFORMS = [
  {
    TYPE: 'Windows_NT',
    ARCHITECTURE: 'x64',
    RUST_TARGET: 'x86_64-pc-windows-gnu',
    BINARY_NAME: `${name}.exe`,
  },
  {
    TYPE: 'Linux',
    ARCHITECTURE: 'x64',
    RUST_TARGET: 'x86_64-unknown-linux-musl',
    BINARY_NAME: name,
  },
  {
    TYPE: 'Darwin',
    ARCHITECTURE: 'x64',
    RUST_TARGET: 'x86_64-apple-darwin',
    BINARY_NAME: name,
  },
  {
    TYPE: 'Darwin',
    ARCHITECTURE: 'arm64',
    RUST_TARGET: 'x86_64-apple-darwin',
    BINARY_NAME: name,
  },
];

const error = message => {
  console.error(message);
  process.exit(1);
};

const getPlatform = () => {
  const type = os.type();
  const architecture = os.arch();

  for (const supportedPlatform of SUPPORTED_PLATFORMS) {
    if (
      type === supportedPlatform.TYPE &&
      architecture === supportedPlatform.ARCHITECTURE
    ) {
      return supportedPlatform;
    }
  }

  error(
    `Platform with type "${type}" and architecture "${architecture}" is not supported by ${name}.\n` +
      `Your system must be one of the following:\n\n${cTable.getTable(
        SUPPORTED_PLATFORMS
      )}`
  );
};

const getBinary = () => {
  const platform = getPlatform();
  const url = `${repository.url}/releases/download/v${version}/${name}_v${version}_${platform.RUST_TARGET}.tar.gz`;

  return binary({ name: platform.BINARY_NAME, url });
};

const install = ({ logLevel } = { logLevel: 'info' }) => {
  const binary = getBinary();

  const { install } = makeInstaller({
    root: __dirname,
    binary,
    logLevel,
    requestOptions: configureProxy(binary.url),
  });

  return install();
};

const run = () => {
  const binary = getBinary();

  runBinary({
    root: __dirname,
    binary,
    requestOptions: configureProxy(binary.url),
  });
};

module.exports = { install, run };
