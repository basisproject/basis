"use strict";

const config = require('../helpers/config');
const Basis = require('lib-basis-client');
Basis.init(config);
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = Basis.transactions
const tx = trans.types;
const bootstrap = Basis.bootstrap;
const proto = Basis.protobuf;
const ResourceTags = Basis.models.resource_tags;

describe('resource tags', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const member_id = uuid();
	const company_id = uuid();
	const product_id = uuid();
	const tag_id = uuid();

	beforeAll((done) => {
		trans.clear_users();
		trans.add_user('root', config.bootstrap_user.pub, config.bootstrap_user.sec);
		bootstrap.load().then(done).catch(done.fail);
	});

	afterAll((done) => {
		bootstrap.unload().then(done).catch(done.fail);
	});

	it('can setup', async () => {
		var res = await trans.send_as('root', tx.company.TxCreatePrivate, {
			id: company_id,
			email: 'wedonthaveanykids@gmail.com',
			name: 'LEONARD\'S IRON MINE',
			founder_member_id: member_id,
			founder_occupation: 'Miner 49er',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('root', tx.product.TxCreate, {
			id: product_id,
			company_id: company_id,
			name: 'iron ore',
			unit: 'MILLIMETER',
			mass_mg: 100.0,
			dimensions: {
				width: 1000,
				height: 1000,
				length: 1000,
			},
			inputs: [],
			effort: {
				time: proto.types.Time.gen('MINUTES'),
				quantity: 6,
			},
			active: true,
			meta: '',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});

	it('can be added', async () => {
		var res = await trans.send_as('root', tx.resource_tag.TxCreate, {
			id: tag_id,
			product_id: product_id,
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('root', tx.resource_tag.TxCreate, {
			id: tag_id,
			product_id: product_id,
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/id already exists/i);

		var res = await trans.send_as('root', tx.resource_tag.TxCreate, {
			id: tag_id,
			product_id: product_id+'z',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/product not found/i);
	});

	it('can be deleted', async () => {
		var res = await trans.send_as('root', tx.resource_tag.TxDelete, {
			id: tag_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('root', tx.resource_tag.TxDelete, {
			id: tag_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/already deleted/i);

		var res = await trans.send_as('root', tx.resource_tag.TxDelete, {
			id: tag_id+'z',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/resource tag not found/i);
	});

	it('can tear down', async () => {
		var res = await trans.send_as('root', tx.product.TxDelete, {
			id: product_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('root', tx.company.TxDelete, {
			id: company_id,
			memo: 'Oh! goingtoblackbear',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});
});


