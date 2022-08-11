#!/usr/bin/env node

const { install: maybeInstall, run } = require('./binary');

maybeInstall({ suppressLogs: true }).then(run);
