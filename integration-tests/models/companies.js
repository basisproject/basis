"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/companies', 'basis.company.Company');
exports.list = list;
exports.get = get;

