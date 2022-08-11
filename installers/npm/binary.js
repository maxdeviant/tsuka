const os = require('os');
const { configureProxy } = require('axios-proxy-builder');
const { Binary } = require('binary-install');
const cTable = require('console.table');

const { name, version, repository } = require('./package.json');

const SUPPORTED_PLATFORMS = [
  {
    TYPE: 'Linux',
    ARCHITECTURE: 'x64',
    RUST_TARGET: 'x86_64-unknown-linux-musl',
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
  const url = `${repository.url}/releases/download/v${version}/v${name}_${version}_${platform.RUST_TARGET}.tar.gz`;

  return new Binary(platform.BINARY_NAME, url);
};

const install = ({ suppressLogs } = { suppressLogs: false }) => {
  const binary = getBinary();
  const proxy = configureProxy(binary.url);

  return binary.install(proxy, suppressLogs);
};

const run = () => {
  const binary = getBinary();
  binary.run();
};

module.exports = { install, run };
