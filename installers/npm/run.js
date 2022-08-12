#!/usr/bin/env node

const { install: maybeInstall, run } = require('./binary');

maybeInstall({ logLevel: 'warn' }).then(run);
