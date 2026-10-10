---
{
 "slug": "wesabe",
 "name": "Wesabe",
 "url": "https://wesabe.com",
 "segment": "personal-finance",
 "status": "shut",
 "founded": "December 2005 (site live November 2006)",
 "hq": "San Francisco, California [unverified]",
 "backing": "About $4.7 million disclosed: $700,000 from O'Reilly AlphaTech Ventures (February 2007) and a $4 million round led by Union Square Ventures (announced 20 June 2007), per TechCrunch, 30 June 2010. Shut down 31 July 2010.",
 "tagline": "A privacy-minded web service for personal finances, 2006 to 2010, that lost to Mint and released its code when it closed.",
 "scope": "Wesabe pulled bank and card statements (as OFX) into a hosted site, let you tag and edit transactions, set spending targets, see trends and swap tips in community groups. Its bank data came through its own uploader software rather than a commercial aggregator. It did no bookkeeping, no business accounting and no bill payment.",
 "domains": [
  "personal",
  "bank"
 ],
 "customers_served": [
  "consumer"
 ],
 "spectrum": {
  "custody": {
   "score": 1,
   "note": "Hosted service held the working copy; you could export, and were told to before data was deleted on 31 July 2010."
  },
  "locality": {
   "score": 1,
   "note": "The uploader ran on your machine, the product ran in Wesabe's cloud; the code became runnable at home only after shutdown."
  },
  "history": {
   "score": 1,
   "note": "Statements accumulated as you uploaded them, but edits and tags changed records in place; no evidence of versioned history."
  },
  "openness": {
   "score": 3,
   "note": "Closed while running; after shutdown the site, engine and uploader were released under Apache-2.0 (archived 8 May 2018)."
  },
  "breadth": {
   "score": 1,
   "note": "Bank and card transactions only."
  },
  "autonomy": {
   "score": 1,
   "note": "By its founder's own account it made users do much of the editing and categorizing work."
  },
  "agent_readiness": {
   "score": 0,
   "note": "Predates agents; whether it had a public developer API is unverified."
  },
  "integration_ease": {
   "score": 1,
   "note": "Per-bank scraping scripts and a downloadable uploader; harder to set up than Mint's aggregator logins."
  },
  "market": {
   "score": 0,
   "note": "Consumers."
  }
 },
 "api": {
  "public": false,
  "style": "None confirmed while operating; the open-sourced SSU uploader exposes a small local HTTP server returning JSON.",
  "auth": "n/a (SSU took bank credentials locally per job)",
  "limits": "n/a",
  "cost": "The service was free to users [unverified]; the code is free under Apache-2.0.",
  "export": "Account data download offered before shutdown (format not confirmed); the open-sourced fixofx tool converts several financial formats to OFX 2.",
  "notes": "Repositories at github.com/wesabe: pfc (Rails front end), brcm-accounts-api (Java transaction and analytics engine), ssu (Server-Side Uploader, XULRunner with per-bank scripts), fixofx (Python), all archived."
 },
 "pricing": {
  "model": "Free consumer web service [unverified]",
  "entry": "Free [unverified]",
  "top": "n/a",
  "notes": "No paid tier confirmed. Funding came from venture capital, not users."
 },
 "agents": {
  "mcp": "n/a",
  "ai_features": "None. Its advice came from community tips and spending analysis, not models.",
  "agent_writes": "n/a",
  "approval": "n/a",
  "notes": "Shut in 2010, long before agents."
 },
 "integration": {
  "scope": "Banks and card issuers through per-institution scripts in its own uploader (count of supported institutions not confirmed).",
  "notes": "SSU drove a scriptable Mozilla browser (XULRunner) with one script per bank, stored by reverse-DNS name (for example com/chase), to log in and download OFX. Linux and macOS only in the open-source release."
 },
 "preference": {
  "scale": "No user counts published; TechCrunch reported traffic fell steadily after its April 2009 iPhone app.",
  "ratings": "None (predates the review sites; no G2, Capterra or app store ratings found).",
  "sentiment": "Devoted early users liked the downloadable import client and the privacy stance (Hacker News thread, 1 October 2010). The market chose Mint: easier setup through Yodlee, automatic categorization and better design. Hedlund's post drew 373 points and 195 comments on Hacker News."
 },
 "timeline": [
  {
   "date": "2005-12",
   "event": "Company established"
  },
  {
   "date": "2006-11",
   "event": "Site goes live"
  },
  {
   "date": "2007-02",
   "event": "$700,000 from O'Reilly AlphaTech Ventures"
  },
  {
   "date": "2007-06-20",
   "event": "$4 million round led by Union Square Ventures announced"
  },
  {
   "date": "2009-04",
   "event": "iPhone app launches; traffic declines afterwards"
  },
  {
   "date": "2009-09",
   "event": "Intuit buys Mint for $170 million [month unverified]"
  },
  {
   "date": "2010-06-30",
   "event": "Shutdown announced; no new accounts; users told to download data"
  },
  {
   "date": "2010-07-31",
   "event": "Accounts tab switched off and data deleted; Groups kept online by a customer"
  },
  {
   "date": "2010-10-01",
   "event": "Marc Hedlund publishes 'Why Wesabe lost to Mint'"
  },
  {
   "date": "2010-2011",
   "event": "Site code released under Apache-2.0 on GitHub [release month unverified]"
  },
  {
   "date": "2018-05-08",
   "event": "GitHub repositories archived read-only"
  },
  {
   "date": "2026-10-10",
   "event": "wesabe.com redirects to due.com"
  }
 ],
 "relation": "cautionary-tale",
 "relation_note": "Wesabe tried to give people a privacy-respecting view of their own money with a local uploader, and lost to a rival that was easier because it handed credentials to an aggregator. It is the clearest early lesson that principle without ease of use does not win, and that a hosted record dies with its host.",
 "borrow": "Per-source scripts in a plain folder layout that contributors can add to; converting messy bank formats to one standard (fixofx); releasing the code and a data download when the service ends; keeping credentials on the owner's machine.",
 "avoid": "Making the owner do the categorizing and editing work; depending on fragile screen-scraping per bank; holding the only working copy in a hosted service that can shut with a month's notice.",
 "sources": [
  {
   "title": "TechCrunch: Online finance startup Wesabe heads to the deadpool (30 June 2010)",
   "url": "https://techcrunch.com/2010/06/30/wesabe-shuts-down/"
  },
  {
   "title": "Wikipedia: Wesabe",
   "url": "https://en.wikipedia.org/wiki/Wesabe"
  },
  {
   "title": "Hacker News: Why Wesabe lost to Mint (1 October 2010)",
   "url": "https://news.ycombinator.com/item?id=1746832"
  },
  {
   "title": "Hacker News: submissions from precipice.org",
   "url": "https://news.ycombinator.com/from?site=precipice.org"
  },
  {
   "title": "GitHub: wesabe organization",
   "url": "https://github.com/wesabe"
  },
  {
   "title": "GitHub: wesabe/ssu",
   "url": "https://github.com/wesabe/ssu"
  },
  {
   "title": "GitHub: wesabe/pfc",
   "url": "https://github.com/wesabe/pfc"
  }
 ],
 "unverified": [
  "The original post-mortem at blog.precipice.org could not be fetched (host no longer resolves; web archive blocked); quotes come from readers quoting it on Hacker News.",
  "HQ in San Francisco and co-founder Jason Knight are from memory, not confirmed in a source fetched here.",
  "Whether Wesabe had a public developer API while operating.",
  "Whether the service was free with no paid tier.",
  "Exact month the code was open-sourced (repositories show only archive dates).",
  "Month of Intuit's Mint acquisition (believed September 2009).",
  "Format of the data download offered at shutdown.",
  "Web search budget was exhausted during this pass; findings rely on directly fetched pages."
 ],
 "researched": "2026-10-10"
}
---

## What it is

Wesabe was a personal finance website that ran from November 2006 to July 2010. It was established in December 2005, analyzed your bank and card transactions, and gave advice on saving money, much of it from community groups where members swapped tips. Its CEO was Marc Hedlund. It is gone: the [shutdown was announced on 30 June 2010](https://techcrunch.com/2010/06/30/wesabe-shuts-down/) and the accounts service ended on 31 July 2010. Today wesabe.com redirects to an unrelated company (checked 10 October 2026).

It matters to ownpurse because of how it lost, and because of what its founder wrote about it.

## How it works

Wesabe did not license a commercial aggregator. It built its own uploader. The open-sourced version, [SSU, the Server-Side Uploader](https://github.com/wesabe/ssu), is a scripted Mozilla browser (XULRunner) with one script per bank, filed by reverse-DNS name, that logs in with your credentials and downloads OFX. It runs a small HTTP server on your machine and answers in JSON. The idea was that bank passwords need not leave your computer, and that a real browser breaks less often than a bare scraper.

On the site you tagged transactions, edited descriptions, set spending targets and watched trends. The transaction and analytics engine ([brcm-accounts-api](https://github.com/wesabe)) and the Rails front end ([pfc](https://github.com/wesabe/pfc)) sat behind it.

## Data and access

The record lived on Wesabe's servers. When it closed, users were told to download their account information by 31 July, after which it would be deleted ([TechCrunch](https://techcrunch.com/2010/06/30/wesabe-shuts-down/)). The company then released the site's code under Apache-2.0, so a person could run it on their own Mac or Linux machine. The repositories, including [fixofx](https://github.com/wesabe), a converter from several financial formats to OFX 2, were archived on 8 May 2018. Whether Wesabe offered a public developer API while it ran is not confirmed here.

## Agents

None. Wesabe predates agents and used no models. Its relevance is the uploader: per-source scripts are exactly the kind of connector an agent can now write in an afternoon.

## Customers

Consumers. No user counts were published. TechCrunch reported that traffic fell steadily after its April 2009 iPhone app. Fans liked the downloadable import client and the privacy stance, as the [Hacker News discussion](https://news.ycombinator.com/item?id=1746832) of the post-mortem (373 points) shows. The market chose Mint, which Intuit bought for $170 million.

## History

Wesabe raised about $4.7 million: $700,000 from O'Reilly AlphaTech Ventures in February 2007 and a $4 million round led by Union Square Ventures in June 2007. When it closed, Hedlund said running a service that holds such sensitive data "with shoestring operations and security staff is not acceptable."

On 1 October 2010 he published "Why Wesabe lost to Mint" on his blog (the original host no longer resolves; it is [indexed on Hacker News](https://news.ycombinator.com/from?site=precipice.org)). As quoted in that thread, his summary was that "Mint focused on making the user do almost no work at all, by automatically editing and categorizing their data," while Wesabe spent its usability effort in the wrong place. Mint used Yodlee; Wesabe had looked at Yodlee in 2006 and found "the company was crumbling," so it built its own aggregation. Readers drew the blunt conclusion that users do not pay for privacy with effort.

## For ownpurse

Wesabe is the warning in ownpurse's own positioning. Owner-held credentials and a principled stance did not save a product that made people do the work. ownpurse's bet is that agents now do that work: they write the connector, categorize, and walk you through the result, so privacy no longer costs effort. Wesabe also shows what a hosted record means at the end: a month's notice and a download button. A file the owner already holds needs no rescue.

What to borrow: per-source scripts in a plain layout that anyone can extend; one standard shape for messy bank formats; credentials that stay on the owner's machine. What to avoid: screen-scraping as the primary path when a bank offers a file or an API, and any design where the owner is the categorization engine.
