"use strict";

const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = require('../helpers/transactions');
const tx = trans.types;
const bootstrap = require('../helpers/bootstrap');
const config = require('../helpers/config');
const Products = require('../models/products');
const proto = require('../helpers/protobuf');

describe('products', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const jerry_user_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	// today, sandra owns the company. jerry is her assistant
	const sandra_user_id = uuid();
	const {publicKey: sandra_pubkey, secretKey: sandra_seckey} = Exonum.keyPair();
	const sandra_email = 'sandra@thatscool.net';

	const company_id = uuid();
	const company_email = 'info@SANDRASwidgets.com';

	const product_id = uuid();
	const product_name = 'Whiffle Widget';
	const product_name2 = 'Serious Widget';

	beforeAll((done) => {
		trans.clear_users();
		trans.add_user('root', config.bootstrap_user.pub, config.bootstrap_user.sec);
		trans.add_user('jerry', jerry_pubkey, jerry_seckey);
		trans.add_user('sandra', sandra_pubkey, sandra_seckey);
		bootstrap.load().then(done).catch(done.fail);
	});

	afterAll((done) => {
		bootstrap.unload().then(done).catch(done.fail);
	});

	it('setup', async () => {
		// no, jerry! stop, jerry!
		var res = await trans.send_as('root', tx.user.TxCreate, {
			id: jerry_user_id,
			pubkey: jerry_pubkey,
			roles: ['User'],
			email: jerry_email,
			name: 'jerry jerjer jordan',
			meta: '{}',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		// create sandra, who will be the company admin
		var res = await trans.send_as('root', tx.user.TxCreate, {
			id: sandra_user_id,
			pubkey: sandra_pubkey,
			roles: ['User'],
			email: sandra_email,
			name: 'Sandra "The Eliminator" Pilkington',
			meta: '{"tester":true}',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.company.TxCreatePrivate, {
			id: company_id,
			email: company_email,
			name: 'SANDRA\'s (NOT Jerry\'s) WIDGETS',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});

	it('creates new products', async () => {
		var res = await trans.send_as('jerry', tx.product.TxCreate, {
			id: product_id,
			company_id: company_id,
			name: 'jerry iz kewl',
			unit: 'MILLIMETER',
			mass_mg: 42,
			dimensions: {
				width: 100,
				height: 100,
				length: 100,
			},
			inputs: [],
			effort: {
				time: proto.types.Time.gen('hours'),
				quantity: 37,
			},
			active: true,
			meta: '',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		var res = await trans.send_as('sandra', tx.company_member.TxCreate, {
			company_id: company_id,
			user_id: jerry_user_id,
			roles: ['ProductAdmin'],
			memo: 'GET TO WORK, JERRY',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.product.TxCreate, {
			id: product_id,
			company_id: company_id,
			name: product_name,
			unit: 'MILLIMETER',
			mass_mg: 42,
			dimensions: {
				width: 100,
				height: 100,
				length: 100,
			},
			inputs: [],
			effort: {
				time: proto.types.Time.map.HOURS,
				quantity: 37,
			},
			active: false,
			meta: '',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var product = await Products.get({id: product_id});
		expect(product.id).toBe(product_id);
		expect(product.company_id).toBe(company_id);
		expect(product.name).toBe(product_name);
		expect(product.active).toBe(false);
	});

	it('can be updated', async () => {
		var res = await trans.send_as('sandra', tx.product.TxUpdate, {
			id: product_id,
			name: product_name2,
			active: true,
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var product = await Products.get({id: product_id});
		expect(product.id).toBe(product_id);
		expect(product.company_id).toBe(company_id);
		expect(product.name).toBe(product_name2);
		expect(product.active).toBe(true);
		expect(product.mass_mg).toBe(42);
		expect(product.inputs).toEqual([]);

		var input_id = uuid();
		var res = await trans.send_as('sandra', tx.product.TxUpdate, {
			id: product_id,
			inputs: [
				{product_id: input_id, quantity: 69},
			],
			active: true,
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var product = await Products.get({id: product_id});
		expect(product.id).toBe(product_id);
		expect(product.company_id).toBe(company_id);
		expect(product.name).toBe(product_name2);
		expect(product.active).toBe(true);
		expect(product.mass_mg).toBe(42);
		expect(product.inputs).toEqual([{product_id: input_id, quantity: 69}]);
	});

	it('can be destroyed', async () => {
		var res = await trans.send_as('jerry', tx.product.TxDelete, {
			id: product_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.company.TxDelete, {
			id: company_id,
			memo: 'Closing the company because JERRY ruined everything UGH',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.user.TxDelete, {
			id: jerry_user_id,
			memo: 'i like capitalism better <@;)',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('sandra', tx.user.TxDelete, {
			id: sandra_user_id,
			memo: 'i guess capitalism truly is the best!',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var product = await Products.get({id: product_id});
		expect(product.id).toBe(product_id);
		expect(product.deleted.seconds).not.toBe(0);
	});
});

