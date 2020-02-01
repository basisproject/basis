"use strict";

const config = require('../helpers/config');
const Basis = require('lib-basis-client');
Basis.init(config);
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = Basis.transactions
const tx = trans.types;
const bootstrap = Basis.bootstrap;
const Products = Basis.models.products;
const proto = Basis.protobuf;

describe('products', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const jerry_user_id = uuid();
	const jerry_member_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	// today, sandra owns the company. jerry is her assistant
	const sandra_user_id = uuid();
	const sandra_member_id = uuid();
	const {publicKey: sandra_pubkey, secretKey: sandra_seckey} = Exonum.keyPair();
	const sandra_email = 'sandra@thatscool.net';

	const company_id = uuid();
	const company_email = 'info@SANDRASwidgets.com';

	const ctag_op_id = uuid();
	const ctag_inv_id = uuid();

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
			cost_tags: [
				{id: ctag_op_id, name: "operating"},
				{id: ctag_inv_id, name: "inventory"},
			],
			founder: {
				member_id: sandra_member_id,
				occupation: 'Widget builder',
				wage: 69.0,
				default_cost_tags: [{id: ctag_op_id, weight: 10}],
			},
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
			cost_tags: [
				{id: ctag_op_id, weight: 1},
				{id: ctag_inv_id, weight: 4},
			],
			active: true,
			meta: '',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		var res = await trans.send_as('sandra', tx.company_member.TxCreate, {
			id: jerry_member_id,
			company_id: company_id,
			user_id: jerry_user_id,
			roles: ['ProductAdmin', 'CostTaggerProduct'],
			occupation: 'Data entry',
			wage: 0.001,
			default_cost_tags: [{id: ctag_op_id, weight: 10}],
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
			cost_tags: [
				{id: ctag_op_id, weight: 3},
				{id: ctag_inv_id, weight: 5},
				{id: 'fake tag. SAD!', weight: 69},
			],
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
		var cost_tags = product.cost_tags.sort((a, b) => a.weight - b.weight);
		expect(cost_tags.length).toBe(2);
		expect(cost_tags[0]).toEqual({id: ctag_op_id, weight: 3});
		expect(cost_tags[1]).toEqual({id: ctag_inv_id, weight: 5});
	});

	it('can be updated', async () => {
		var res = await trans.send_as('sandra', tx.product.TxUpdate, {
			id: product_id,
			name: product_name2,
			active: true,
			cost_tags: [
				{id: ctag_inv_id, weight: 42},
			],
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var product = await Products.get({id: product_id});
		expect(product.id).toBe(product_id);
		expect(product.company_id).toBe(company_id);
		expect(product.name).toBe(product_name2);
		expect(product.active).toBe(true);
		expect(product.mass_mg).toBe(42);
		expect(product.cost_tags).toEqual([{id: ctag_inv_id, weight: 42}]);

		var input_id = uuid();
		var res = await trans.send_as('sandra', tx.product.TxUpdate, {
			id: product_id,
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

