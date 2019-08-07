"use strict";

const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = require('../helpers/transactions');
const tx = trans.types;
const bootstrap = require('../helpers/bootstrap');
const config = require('../helpers/config');
const Members = require('../models/companies-members');

describe('company members', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const jerry_user_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	// a sidekick for jerjer
	const sandra_user_id = uuid();
	const {publicKey: sandra_pubkey, secretKey: sandra_seckey} = Exonum.keyPair();
	const sandra_email = 'sandra@thatscool.net';

	const company_id = uuid();
	const company_email = 'info@jerryswidgets.com';

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

	it('can be added to companies by owners', async () => {
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

		// create sandra, who will be used to test permissions
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

		var res = await trans.send_as('jerry', tx.company.TxCreatePrivate, {
			id: company_id,
			email: company_email,
			name: 'jerry\'s WIDGETS',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.company_member.TxCreate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['ProductAdmin'],
			memo: 'Sandra seems trustworthy',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});

	it('can be destroyed', async () => {
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
		var res = await trans.send_as('sandra', tx.user.TxDelete, {
			id: sandra_user_id,
			memo: 'i go where jerry goes',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});
});

