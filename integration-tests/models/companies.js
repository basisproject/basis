"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/companies', 'factor.company.Company');
exports.list = list;
exports.get = get;

