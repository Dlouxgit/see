#!/usr/bin/env node

const rust_test = require('../index.node')

try {
    rust_test.parser()
} catch (e) {
    console.log(e)
}