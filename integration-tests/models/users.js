"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/users', 'basis.user.User');
exports.list = list;
exports.get = get;

