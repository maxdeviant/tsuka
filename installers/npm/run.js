#!/usr/bin/env node

const { install: maybeInstall, run } = require('./binary');

maybeInstall(true).then(run);
