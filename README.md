# IronCalc for Nextcloud

[![MIT licensed][mit-badge]][mit-url]
[![Apache 2.0 licensed][apache-badge]][apache-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/ironcalc/IronCalc/blob/main/LICENSE-MIT

[apache-badge]: https://img.shields.io/badge/License-Apache_2.0-blue.svg
[apache-url]: https://github.com/ironcalc/IronCalc/blob/main/LICENSE-Apache-2.0

[Nextcloud](https://github.com/nextcloud) app to extend it with [IronCalc](https://github.com/ironcalc) for working with spreadsheets. Current stage: proof of concept.

## Development build:

1. Start an instance of Nextcloud to test against:
   ```
   docker run --name nextcloud-test \
      -v nextcloud-test:/var/www/html \
      -e 'SQLITE_DATABASE=db.sqlite' \
      -e 'NEXTCLOUD_ADMIN_USER=admin' \
      -e 'NEXTCLOUD_ADMIN_PASSWORD=admin' \
      -p 2180:80 \
      nextcloud:apache
   ```
2. Upload a spreadsheet:
   ```
   curl -u admin:admin -T test/fixtures/mortgage_calculator.xlsx \
      http://localhost:2180/remote.php/dav/files/admin/Documents/
   ```
3. Run the proxy: `caddy run`
4. In the server folder run `cargo run`
5. In the frontend folder run `npm install && npm run dev`
6. Manually load the file (by id) in the frontend
      1. Open Nextcload in your browser: http://localhost:2180/apps/files/files?dir=/Documents
      2. Right-click the XLSX file, click 'Open details'
      3. Read the (numeric) file id from the url, eg. 124 in `http://localhost:2180/apps/files/files/124?dir=/Documents&opendetails=true`.
      4. Pass the file id as parameter to the frontend: `http://localhost:2080/?fileIds=124`


# TODO

Automated testing, proper error handling and messages, load integration from within Nextcloud, make it work as well as the markdown editor does, collaboration, package as Nextcloud App, ...

# License

Based on code from [the IronCalc webapp](https://github.com/ironcalc/IronCalc/tree/main/webapp) (in addition to depending on the IronCalc software libraries.)

Licensed under either of

* [MIT license](LICENSE-MIT)
* [Apache license, version 2.0](LICENSE-Apache-2.0)

at your option.

# Funding

[This project](https://nlnet.nl/project/IronCalc-NC/) was funded through the [NGI0 Commons Fund](https://nlnet.nl/commonsfund), a fund established by [NLnet](https://nlnet.nl/) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu/) programme, under the aegis of [DG Communications Networks, Content and Technology](https://commission.europa.eu/about-european-commission/departments-and-executive-agencies/communications-networks-content-and-technology_en) under grant agreement No [101135429](https://cordis.europa.eu/project/id/101135429). Additional funding is made available by the [Swiss State Secretariat for Education, Research and Innovation](https://www.sbfi.admin.ch/sbfi/en/home.html) (SERI).
