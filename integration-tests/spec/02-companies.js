"use strict";

const config = require('../helpers/config');
const Basis = require('lib-basis-client');
Basis.init(config);
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = Basis.transactions
const tx = trans.types;
const bootstrap = Basis.bootstrap;
const Companies = Basis.models.companies;

describe('companies', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	const jerry_user_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	// a tormentor for jerry
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

	it('can be created', async () => {
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
		expect(res.description).toBeFalsy();

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
		expect(res.description).toBeFalsy();

		// bad email should FAIL
		var res = await trans.send_as('jerry', tx.company.TxCreatePrivate, {
			id: company_id,
			email: 'sasssssafrassss',
			name: 'jerry\'s WIDGETS',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description.match(/invalid email/i)).toBeTruthy();

		var res = await trans.send_as('jerry', tx.company.TxCreatePrivate, {
			id: company_id,
			email: company_email,
			name: 'jerry\'s WIDGETS',
			created: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var company = await Companies.get({id: company_id});
		expect(company.id).toBe(company_id);
		expect(company.email).toBe(company_email);
		expect(company.name).toBe('jerry\'s WIDGETS');
	});

	it('can be listed', async () => {
		let companies = await Companies.list({});
		function to_date(blockdate) {
			var ms = blockdate.seconds * 1000;
			ms += Math.round(blockdate.nanos / 1000000);
			return new Date(ms);
		}
		companies.sort((a, b) => to_date(a.created) - to_date(b.created));
		expect(companies.length).toBe(1);
		expect(companies[0].email).toBe(company_email);
	});

	it('can be updated', async () => {
		var res = await trans.send_as('jerry', tx.company.TxUpdate, {
			id: company_id,
			email: '',
			name: 'Widget emporium',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var company = await Companies.get({id: company_id});
		expect(company.id).toBe(company_id);
		expect(company.email).toBe(company_email);
		expect(company.name).toBe('Widget emporium');

		var res = await trans.send_as('sandra', tx.company.TxUpdate, {
			id: company_id,
			email: '',
			name: 'HA! Jerry is terrible at business',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);
	});

	it('can have their type updated', async () => {
		var company = await Companies.get({id: company_id});
		expect(company.ty).toBe('PRIVATE');

		// jerry cannot set his own type
		var res = await trans.send_as('jerry', tx.company.TxSetType, {
			id: company_id,
			ty: 'SYNDICATE',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		// ...and sandra certainly should not be able to
		var res = await trans.send_as('sandra', tx.company.TxSetType, {
			id: company_id,
			ty: 'SYNDICATE',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		// root can, tho
		var res = await trans.send_as('root', tx.company.TxSetType, {
			id: company_id,
			ty: 'MY CUSTOM TYPE',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var company = await Companies.get({id: company_id});
		expect(company.ty).toBe('UNKNOWN');

		var res = await trans.send_as('root', tx.company.TxSetType, {
			id: company_id,
			ty: 'SYNDICATE',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var company = await Companies.get({id: company_id});
		expect(company.ty).toBe('SYNDICATE');

		var res = await trans.send_as('root', tx.company.TxSetType, {
			id: company_id,
			ty: 'PRIVATE',
			updated: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var company = await Companies.get({id: company_id});
		expect(company.ty).toBe('PRIVATE');
	});

	it('can be destroyed', async () => {
		// sandra, at it again
		var res = await trans.send_as('sandra', tx.company.TxDelete, {
			id: company_id,
			memo: 'Let\'s see how well your company does after I DELETE IT, JERRY',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description).toMatch(/insufficient priv/i);

		var res = await trans.send_as('jerry', tx.company.TxDelete, {
			id: company_id,
			memo: 'Nobody buys my widgets...',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);

		var res = await trans.send_as('jerry', tx.company.TxDelete, {
			id: company_id,
			memo: 'delete this twice?',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(false);
		expect(res.description.match(/not found/i)).toBeTruthy();

		var res = await trans.send_as('jerry', tx.user.TxDelete, {
			id: jerry_user_id,
			memo: 'i like capitalism better',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
		var res = await trans.send_as('sandra', tx.user.TxDelete, {
			id: sandra_user_id,
			memo: 'if i cannot torment jerry, my life has no meaning',
			deleted: new Date().toISOString(),
		});
		expect(res.success).toBe(true);
	});
});

