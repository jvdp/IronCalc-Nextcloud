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
      -p 2100:80 \
      nextcloud:apache
   ```
2. Upload a spreadsheet:
   ```
   curl -u admin:admin -T test/fixtures/mortgage_calculator.xlsx \
      http://localhost:2100/remote.php/dav/files/admin/Documents/
   ```
3. Run the proxy: `caddy run`
4. In the server folder run `cargo run`
5. In the frontend folder run `npm install && npm run dev`
6. Register the Rust server with the Nextcloud App API: `make register`
7. Open Nextcload in your browser: http://localhost:2180/apps/files/files?dir=/Documents
8. Right-click the XLSX file (or click the ellipses)
9. Click "Open with IronCalc"


# TODO

- use `files_action_handler` request payload to replace Webdav search call 
- configure Nextclouds Apache to strip CSP headers instead of using Caddy
- automated testing
- proper error handling and messages
- collaboration
- release build
- package as Nextcloud App
- ultimately: make it work as well as the markdown editor does.

# License

Includes a vendored package of the IronCalc Workbook component.

Based on code from [the IronCalc webapp](https://github.com/ironcalc/IronCalc/tree/main/webapp) (in addition to depending on the IronCalc software libraries.)

Licensed under either of

* [MIT license](LICENSE-MIT)
* [Apache license, version 2.0](LICENSE-Apache-2.0)

at your option.

# Funding

[This project](https://nlnet.nl/project/IronCalc-NC/) was funded through the [NGI0 Commons Fund](https://nlnet.nl/commonsfund), a fund established by [NLnet](https://nlnet.nl/) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu/) programme, under the aegis of [DG Communications Networks, Content and Technology](https://commission.europa.eu/about-european-commission/departments-and-executive-agencies/communications-networks-content-and-technology_en) under grant agreement No [101135429](https://cordis.europa.eu/project/id/101135429). Additional funding is made available by the [Swiss State Secretariat for Education, Research and Innovation](https://www.sbfi.admin.ch/sbfi/en/home.html) (SERI).
