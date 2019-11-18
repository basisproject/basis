"use strict";

const config = require('../helpers/config');
const Basis = require('lib-basis-client');
Basis.init(config);
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = Basis.transactions
const tx = trans.types;
const bootstrap = Basis.bootstrap;
const Members = Basis.models.companies_members;

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
			founder_occupation: 'Widget builder',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.company_member.TxCreate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['ProductAdmin'],
			occupation: 'Apprentice Widget Builder',
			memo: 'Sandra seems trustworthy',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var sandra = await Members.get({company_id: company_id, user_id: sandra_user_id});
		expect(sandra.user_id).toBe(sandra_user_id);
		expect(sandra.roles).toEqual(['ProductAdmin']);
		expect(sandra.occupation).toBe('Apprentice Widget Builder');
	});

	it('enforces permissions and ownership', async () => {
		var res = await trans.send_as('sandra', tx.company.TxDelete, {
			id: company_id,
			memo: 'WhOoOoPs!! ;)',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		var res = await trans.send_as('sandra', tx.company_member.TxDelete, {
			company_id: company_id,
			user_id: sandra_user_id,
			memo: 'wen i think about u i delete myself',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		var res = await trans.send_as('sandra', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['Owner'],
			occupation: 'Master Widget Builder',
			memo: 'WhOoOoPs!! ;)',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		var res = await trans.send_as('jerry', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['MemberAdmin'],
			occupation: 'Widget Builder',
			memo: 'be careful, sandra',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var sandra = await Members.get({company_id: company_id, user_id: sandra_user_id});
		expect(sandra.roles).toEqual(['MemberAdmin']);
		expect(sandra.occupation).toBe('Widget Builder');

		var res = await trans.send_as('sandra', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['ProductAdmin'],
			memo: 'wait does this work?',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var sandra = await Members.get({company_id: company_id, user_id: sandra_user_id});
		expect(sandra.roles).toEqual(['ProductAdmin']);
		expect(sandra.occupation).toBe('Widget Builder');

		var res = await trans.send_as('sandra', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['MemberAdmin'],
			memo: 'hahaha bye jerry',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);
	});

	it('enforces at least one owner', async () => {
		var res = await trans.send_as('jerry', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: jerry_user_id,
			roles: ['MemberAdmin'],
			memo: 'too much responsibility',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/company must have at least one owner/i);

		var res = await trans.send_as('jerry', tx.company_member.TxDelete, {
			company_id: company_id,
			user_id: jerry_user_id,
			memo: 'i quit',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/company must have at least one owner/i);

		var res = await trans.send_as('jerry', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['Owner'],
			memo: 'giving sandra full control',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: sandra_user_id,
			roles: ['Admin'],
			memo: 'wait how do i use this??',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('sandra', tx.company_member.TxUpdate, {
			company_id: company_id,
			user_id: jerry_user_id,
			roles: ['Admin'],
			memo: 'can i be owner?',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/company must have at least one owner/i);
	});

	it('can be destroyed', async () => {
		var res = await trans.send_as('jerry', tx.company_member.TxDelete, {
			company_id: company_id,
			user_id: sandra_user_id,
			memo: 'going to have to let you go, sandra',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

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

