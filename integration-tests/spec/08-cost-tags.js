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
const CostTags = Basis.models.cost_tags;
const Timestamp = Basis.protobuf.types.Timestamp;

describe('cost tags', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const member_id = uuid();
	const company_id = uuid();
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
			name: 'HOH!! BLACK BEAR',
			founder_member_id: member_id,
			founder_occupation: 'Professional Gambler',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});

	it('can be added', async () => {
		var res = await trans.send_as('root', tx.cost_tag.TxCreate, {
			id: tag_id,
			company_id: company_id,
			name: "Labor costs",
			active: true,
			meta: '{}',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		expect(res.description).toBeFalsy();

		var tag = await CostTags.get({id: tag_id});
		expect(tag.name).toBe('Labor costs');
		expect(tag.active).toBe(true);
		expect(tag.meta).toBe('{}');
	});

	it('can be updated', async () => {
		var res = await trans.send_as('root', tx.cost_tag.TxUpdate, {
			id: tag_id,
			name: "Gambling costs",
			active: false,
			meta: '{"proceeds_used_for":"chick flick, my kinda movie"}',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		expect(res.description).toBeFalsy();

		var tag = await CostTags.get({id: tag_id});
		expect(tag.name).toBe('Gambling costs');
		expect(tag.active).toBe(false);
		expect(tag.meta).toBe('{"proceeds_used_for":"chick flick, my kinda movie"}');
		expect(Timestamp.from(tag.deleted).toISOString()).toBe(new Date(0).toISOString());
	});

	it('can be deleted', async () => {
		var del = new Date().toISOString()
		var res = await trans.send_as('root', tx.cost_tag.TxDelete, {
			id: tag_id,
			deleted: del,
		});
		expect(res.success).toBe(true);
		expect(res.description).toBeFalsy();

		var tag = await CostTags.get({id: tag_id});
		expect(Timestamp.from(tag.deleted).toISOString()).toBe(del);

		var res = await trans.send_as('root', tx.cost_tag.TxDelete, {
			id: tag_id,
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/already deleted/i);

		var res = await trans.send_as('root', tx.cost_tag.TxDelete, {
			id: tag_id.replace(/[a-f0-9]/gi, '0'),
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/cost tag not found/i);
	});

	it('can tear down', async () => {
		var res = await trans.send_as('root', tx.company.TxDelete, {
			id: company_id,
			memo: 'Hoh! goinblackbear',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});
});



