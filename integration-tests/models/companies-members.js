"use strict";

const standard_api = require('../helpers/standard-api');

const {list, get} = standard_api.generate('/companies/members', 'basis.company_member.CompanyMember');
exports.list = list;
exports.get = get;


