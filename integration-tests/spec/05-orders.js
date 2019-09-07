"use strict";

const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = require('../helpers/transactions');
const tx = trans.types;
const bootstrap = require('../helpers/bootstrap');
const config = require('../helpers/config');
const Orders = require('../models/orders');
const proto = require('../helpers/protobuf');

describe('orders', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const jerry_user_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	const sandra_user_id = uuid();
	const {publicKey: sandra_pubkey, secretKey: sandra_seckey} = Exonum.keyPair();
	const sandra_email = 'sandra@thatscool.net';

	const order_id = uuid();

	const company1_id = uuid();
	const company1_email = 'info@SANDRASwidgets.com';
	const company2_id = uuid();
	const company2_email = 'info@jerryswidgetco.com';

	const company_shipping_id = uuid();

	const product1_id = uuid();
	const product1_name = 'Basic Widget';
	const variant1_id = uuid();
	const product2_id = uuid();
	const product2_name = 'Advanced Widget';
	const variant2_id = uuid();

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
			id: company1_id,
			email: company1_email,
			name: 'SANDRA\'s (NOT Jerry\'s) WIDGETS',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('jerry', tx.company.TxCreatePrivate, {
			id: company2_id,
			email: company2_email,
			name: 'jerry\'s resold widgets',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('jerry', tx.company.TxCreatePrivate, {
			id: company_shipping_id,
			email: 'shipping@jerry.net',
			name: 'jerry\'s logistix',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.product.TxCreate, {
			id: product1_id,
			company_id: company1_id,
			name: product1_name,
			meta: '',
			active: true,
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('sandra', tx.product.TxSetVariant, {
			id: product1_id,
			variant: {
				id: variant1_id,
				name: 'Big widget',
				unit: proto.types.Unit.gen('MILLIMETER'),
				mass_mg: 2.4,
				dimensions: {
					width: 1000,
					height: 1000,
					length: 1000,
				},
				inputs: [],
				options: {
					'size': 'Big',
				},
				effort: {
					time: proto.types.Time.gen('MINUTES'),
					quantity: 6,
				},
				active: true,
				meta: '{}',
			},
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.product.TxCreate, {
			id: product2_id,
			company_id: company1_id,
			name: product2_name,
			meta: '',
			active: true,
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('sandra', tx.product.TxSetVariant, {
			id: product2_id,
			variant: {
				id: variant2_id,
				name: 'Small widget',
				unit: proto.types.Unit.gen('MILLIMETER'),
				mass_mg: 1.4,
				dimensions: {
					width: 100,
					height: 100,
					length: 100,
				},
				inputs: [],
				options: {
					'size': 'Big',
				},
				effort: {
					time: proto.types.Time.gen('MINUTES'),
					quantity: 2,
				},
				active: true,
				meta: '{}',
			},
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});

	it('can be created', async () => {
		const order = {
			id: order_id,
			company_id_from: company2_id,
			company_id_to: company1_id,
			products: [{
				product_id: product1_id,
				product_variant_id: variant1_id,
				quantity: 3,
			}, {
				product_id: product2_id,
				product_variant_id: variant2_id,
				quantity: 6,
			}],
			created: new Date().toISOString(),
		}
		// no, sandra. you cannot order your own product from jerry's company
		var res = await trans.send_as('sandra', tx.order.TxCreate, order);
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		order.created = new Date().toISOString();
		var res = await trans.send_as('jerry', tx.order.TxCreate, order);
		expect(res.success).toBe(true);

		var ord = await Orders.get({id: order_id});
		expect(ord.products[0].product_id).toBe(product1_id);
		expect(ord.products[0].product_variant_id).toBe(variant1_id);
		expect(ord.products[0].quantity).toBe(3);
		expect(ord.products[1].product_id).toBe(product2_id);
		expect(ord.products[1].product_variant_id).toBe(variant2_id);
		expect(ord.products[1].quantity).toBe(6);
		expect(ord.process_status).toBe('NEW');
	});

	it('can be updated', async () => {
		var res = await trans.send_as('sandra', tx.order.TxUpdateStatus, {
			id: order_id,
			process_status: 'ACCEPTED',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var ord = await Orders.get({id: order_id});
		expect(ord.process_status).toBe('ACCEPTED');
	});

	it('destroys', async () => {
		var res = await trans.send_as('sandra', tx.product.TxDelete, {
			id: product1_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('sandra', tx.product.TxDelete, {
			id: product2_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.company.TxDelete, {
			id: company1_id,
			memo: 'Leaving because the gulags don\'t have a day spa',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.company.TxDelete, {
			id: company2_id,
			memo: 'Leaving because not enough gulags',
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
	});
});

