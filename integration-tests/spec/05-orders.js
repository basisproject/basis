"use strict";

const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = require('../helpers/transactions');
const tx = trans.types;
const bootstrap = require('../helpers/bootstrap');
const config = require('../helpers/config');
const Orders = require('../models/orders');

describe('orders', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;
});

