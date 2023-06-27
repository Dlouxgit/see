#!/usr/bin/env node

const seeker = require('../index.node')

const see = async () => {
    try {
        await seeker.parser()
    } catch (e) {
        console.log(e)
    }
}

see()