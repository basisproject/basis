"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/orders', 'basis.order.Order');
exports.list = list;
exports.get = get;

