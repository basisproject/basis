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
	const company_email = 'info@jerryswidgets.com';

	const product_id = uuid();
	const product_name = 'Whiffle Widget';
	const product_name2 = 'Serious Widget';
	const variant_id = uuid();

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
			meta: '{}',
			active: true,
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
			meta: '',
			active: false,
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
			meta: '',
			active: true,
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var product = await Products.get({id: product_id});
		expect(product.id).toBe(product_id);
		expect(product.company_id).toBe(company_id);
		expect(product.name).toBe(product_name2);
		expect(product.active).toBe(true);
	});

	it('can manage options', async () => {
		var product = await Products.get({id: product_id});
		expect(Object.keys(product.options).length).toBe(0);

		var res = await trans.send_as('jerry', tx.product.TxSetOption, {
			id: product_id,
			name: 'size',
			title: 'WiDgEt SiZe',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var product = await Products.get({id: product_id});
		expect(product.options['size']).toBe('WiDgEt SiZe');

		// fixing jerry's stupidness
		var res = await trans.send_as('sandra', tx.product.TxSetOption, {
			id: product_id,
			name: 'size',
			title: 'Size',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var product = await Products.get({id: product_id});
		expect(product.options['size']).toBe('Size');

		var res = await trans.send_as('jerry', tx.product.TxSetOption, {
			id: product_id,
			name: 'color',
			title: 'Color',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var product = await Products.get({id: product_id});
		expect(product.options['size']).toBe('Size');
		expect(product.options['color']).toBe('Color');
		expect(Object.keys(product.options).length).toBe(2);

		var res = await trans.send_as('jerry', tx.product.TxRemoveOption, {
			id: product_id,
			name: 'size',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var product = await Products.get({id: product_id});
		expect(product.options['size']).toBeUndefined();
		expect(product.options['color']).toBe('Color');
		expect(Object.keys(product.options).length).toBe(1);
	});

	it('can manage variants', async () => {
		var input = {
			product_variant_id: uuid(),
			quantity: 6.0,
		};
		var res = await trans.send_as('jerry', tx.product.TxSetVariant, {
			id: product_id,
			variant: {
				id: variant_id,
				name: 'Red Widget',
				unit: proto.types.Unit.gen('MILLIMETER'),
				mass_mg: 2.4,
				dimensions: {
					width: 1000,
					height: 1000,
					length: 1000,
				},
				inputs: [
					input,
				],
				options: {
					'color': 'Red',
					'size': 'Large',
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
		var product = await Products.get({id: product_id});
		var variant = product.variants[variant_id];
		expect(variant.id).toBe(variant_id);
		expect(variant.product_id).toBe(product_id);
		expect(variant.name).toBe('Red Widget');
		expect(variant.options.color).toBe('Red');
		expect(variant.active).toBe(true);
		expect(variant.meta).toBe('{}');

		var res = await trans.send_as('jerry', tx.product.TxUpdateVariant, {
			id: product_id,
			variant_id: variant_id,
			name: 'Reddy McWidget',
			active: false,
			meta: '',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var product = await Products.get({id: product_id});
		var variant = product.variants[variant_id];
		expect(variant.id).toBe(variant_id);
		expect(variant.product_id).toBe(product_id);
		expect(variant.name).toBe('Reddy McWidget');
		expect(variant.options.color).toBe('Red');
		expect(variant.active).toBe(false);
		expect(variant.meta).toBe('{}');
		// TODO: fix date process (or at least compensate here)
		expect(proto.types.Timestamp.from(variant.deleted).getTime()).toBe(0);

		var delete_date = new Date().toISOString();
		var res = await trans.send_as('jerry', tx.product.TxRemoveVariant, {
			id: product_id,
			variant_id: variant_id,
			updated: delete_date,
		});
		expect(res.success).toBe(true);
		var product = await Products.get({id: product_id});
		var variant = product.variants[variant_id];
		expect(variant.id).toBe(variant_id);
		expect(variant.product_id).toBe(product_id);
		expect(variant.name).toBe('Reddy McWidget');
		expect(variant.options.color).toBe('Red');
		expect(variant.active).toBe(false);
		expect(variant.meta).toBe('{}');
		expect(proto.types.Timestamp.from(variant.deleted)).toEqual(new Date(delete_date));
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
		expect(product.variants[variant_id].id).toBe(variant_id);
		expect(product.deleted.seconds).not.toBe(0);
	});
});

