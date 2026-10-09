
DROP VIEW IF EXISTS current;
CREATE VIEW current AS
  SELECT v.* FROM versions v
  JOIN (SELECT tenant_id, entity, xero_id, MAX(version) AS mv FROM versions
        GROUP BY tenant_id, entity, xero_id) m
    ON v.tenant_id = m.tenant_id AND v.entity = m.entity
   AND v.xero_id = m.xero_id AND v.version = m.mv;

DROP VIEW IF EXISTS accounts;
CREATE VIEW accounts AS
  SELECT tenant_id, xero_id AS account_id,
         json_extract(payload,'$.Code') AS code, json_extract(payload,'$.Name') AS name,
         json_extract(payload,'$.Type') AS type, json_extract(payload,'$.Class') AS class,
         json_extract(payload,'$.Status') AS status,
         json_extract(payload,'$.SystemAccount') AS system_account,
         json_extract(payload,'$.BankAccountNumber') AS bank_account_number,
         json_extract(payload,'$.TaxType') AS tax_type, observed_at
  FROM current WHERE entity = 'Accounts';

-- Document date: DateString when present, else the /Date(ms)/ field (ManualJournals, Payments).
DROP VIEW IF EXISTS documents;
CREATE VIEW documents AS
  SELECT tenant_id, entity, xero_id, status,
         COALESCE(substr(json_extract(payload,'$.DateString'),1,10),
                  date(CAST(substr(json_extract(payload,'$.Date'),7,13) AS INTEGER)/1000,'unixepoch')) AS date,
         payload, xero_updated_utc, observed_at
  FROM current;

DROP VIEW IF EXISTS bank_transactions;
CREATE VIEW bank_transactions AS
  SELECT tenant_id, xero_id AS bank_transaction_id, date,
         json_extract(payload,'$.Type') AS type, status,
         json_extract(payload,'$.BankAccount.Name') AS bank_name,
         json_extract(payload,'$.BankAccount.AccountID') AS bank_account_id,
         json_extract(payload,'$.Contact.Name') AS contact,
         json_extract(payload,'$.Reference') AS reference,
         json_extract(payload,'$.IsReconciled') AS is_reconciled,
         xero_updated_utc, observed_at
  FROM documents WHERE entity = 'BankTransactions';

DROP VIEW IF EXISTS manual_journals;
CREATE VIEW manual_journals AS
  SELECT tenant_id, xero_id AS manual_journal_id, date, status,
         json_extract(payload,'$.Narration') AS narration,
         json_extract(payload,'$.ShowOnCashBasisReports') AS show_on_cash,
         xero_updated_utc, observed_at
  FROM documents WHERE entity = 'ManualJournals';
