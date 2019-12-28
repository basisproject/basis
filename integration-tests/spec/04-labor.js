"use strict";

const config = require('../helpers/config');
const Basis = require('lib-basis-client');
Basis.init(config);
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = Basis.transactions
const tx = trans.types;
const bootstrap = Basis.bootstrap;
const Labor = Basis.models.labor;
const Timestamp = Basis.protobuf.types.Timestamp;

describe('labor', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const jerry_user_id = uuid();
	const jerry_member_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	const company_id = uuid();
	const company_email = 'info@jerryswidgets.com';

	const labor1_id = uuid();
	const labor2_id = uuid();

	beforeAll((done) => {
		trans.clear_users();
		trans.add_user('root', config.bootstrap_user.pub, config.bootstrap_user.sec);
		trans.add_user('jerry', jerry_pubkey, jerry_seckey);
		bootstrap.load().then(done).catch(done.fail);
	});

	afterAll((done) => {
		bootstrap.unload().then(done).catch(done.fail);
	});

	it('can setup', async () => {
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

		var res = await trans.send_as('jerry', tx.company.TxCreatePrivate, {
			id: company_id,
			email: company_email,
			name: 'jerry\'s WIDGETS',
			founder_member_id: jerry_member_id,
			founder_occupation: 'Widget builder',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});

	it('can clock in', async () => {
		var now1 = new Date().toISOString();
		var res = await trans.send_as('jerry', tx.labor.TxCreate, {
			id: labor1_id,
			company_id: company_id,
			user_id: jerry_user_id,
			created: now1,
		});
		expect(res.success).toBe(true);

		var labor = await Labor.get({id: labor1_id});
		expect(Timestamp.from(labor.created).toISOString()).toBe(now1);
		expect(Timestamp.from(labor.start).toISOString()).toBe(now1);
		expect(Timestamp.from(labor.end).getTime()).toBe(0);

		var now2 = new Date().toISOString();
		var res = await trans.send_as('jerry', tx.labor.TxCreate, {
			id: labor2_id,
			company_id: company_id,
			user_id: jerry_user_id,
			created: now2,
		});
		expect(res.success).toBe(true);

		var labor = await Labor.get({id: labor2_id});
		expect(Timestamp.from(labor.created).toISOString()).toBe(now2);
		expect(Timestamp.from(labor.start).toISOString()).toBe(now2);
		expect(Timestamp.from(labor.end).getTime()).toBe(0);
	});

	it('can clock out', async () => {
		var now1 = new Date().toISOString();
		var res = await trans.send_as('jerry', tx.labor.TxSetTime, {
			id: labor1_id,
			start: new Date(0).toISOString(),
			end: now1,
			updated: now1,
		});
		expect(res.success).toBe(true);

		var labor = await Labor.get({id: labor1_id});
		expect(labor.created).toEqual(labor.start);
		expect(Timestamp.from(labor.end).toISOString()).toBe(now1);

		var now2 = new Date(new Date().getTime() - (3600 * 6 * 1000)).toISOString();
		var now3 = new Date().toISOString();

		var res = await trans.send_as('jerry', tx.labor.TxSetTime, {
			id: labor2_id,
			start: now2,
			end: now3,
			updated: now3,
		});
		expect(res.success).toBe(true);

		var labor = await Labor.get({id: labor2_id});
		expect(labor.created).not.toEqual(labor.start);
		expect(Timestamp.from(labor.start).toISOString()).toBe(now2);
		expect(Timestamp.from(labor.end).toISOString()).toBe(now3);
	});

	it('can tear down', async () => {
		var res = await trans.send_as('jerry', tx.company.TxDelete, {
			id: company_id,
			memo: 'Nobody buys my widgets...',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.user.TxDelete, {
			id: jerry_user_id,
			memo: 'i like capitalism better <;)',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});
});

