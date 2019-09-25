"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/labor', 'basis.labor.Labor');
exports.list = list;
exports.get = get;

