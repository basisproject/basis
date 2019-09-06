"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/products', 'basis.product.Product');
exports.list = list;
exports.get = get;

