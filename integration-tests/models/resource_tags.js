"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/resource-tags', 'basis.resource_tag.ResourceTag');
exports.list = list;
exports.get = get;

