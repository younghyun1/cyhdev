# Building an unnecessarily fast visitor map with a local Geo-IP database

During the process of constructing this website, I recalled in the hazy depths of my memory (being sick several times during the pandemic seems to have significantly messed with my recollections of life before 2020, which is a blessing of sorts) that many blogs in the mid-late 2000s and early 2010s had a visitor tracker board of sorts, where you could geolocate visitors to your thing and pin them on a map. 

My first instinct was to ignore the prebuilt widgets or whatever and try to make that from scratch myself as much as possible, because I wanted to test out my nonexistent front-end chops as well as my ability to acquire and deploy interesting datasets and serve them in a performant manner. No external API calls!

**1) Acquiring a geo-IP database**

After googling around for a bit, I found that the good fellows over at [IP2Location](https://lite.ip2location.com/ip2location-lite) had a nice set of free databases available that I could use for personal purposes - I opted for the IP2Location™ LITE IP-COUNTRY-REGION-CITY-LATITUDE-LONGITUDE-ZIPCODE Database, which has two versions for IPv4 and IPv6 respectively.

Compressed, they are 47.3MB and 124.8MB respectively; decompressed, they are 312.5MB and 794.2MB in CSV format. The files look something like this:

```csv
"0","16777215","-","-","-","-","0.000000","0.000000","-"
"16777216","16777471","AU","Australia","Queensland","Brisbane","-27.467540","153.028090","4000"
"16777472","16778239","CN","China","Fujian","Fuzhou","26.061390","119.306110","350004"
"16778240","16779007","AU","Australia","Victoria","Melbourne","-37.814007","144.963171","3000"
"16779008","16779263","AU","Australia","Queensland","Warren","-23.500000","150.283330","4702"
"16779264","16781311","CN","China","Guangdong","Guangzhou","23.127361","113.264570","510030"
"16781312","16785407","JP","Japan","Tokyo","Tokyo","35.689497","139.692317","100-0000"
"16785408","16793599","CN","China","Guangdong","Guangzhou","23.127361","113.264570","510030"
"16793600","16793855","JP","Japan","Tokyo","Tokyo","35.689497","139.692317","100-0000"
"16793856","16794367","JP","Japan","Hiroshima","Hiroshima","34.385868","132.455433","730-0011"
"16794368","16794623","JP","Japan","Hiroshima","Nuno","34.533330","132.400000","730-0011"
"16794624","16794879","JP","Japan","Miyagi","Sendai","38.266990","140.867133","980-0802"
"16794880","16795391","JP","Japan","Hiroshima","Hiroshima","34.385868","132.455433","730-0011"
```

An IP address is represented here as an integer. Each row describes an inclusive range, followed by the location IP2Location associates with that range. IPv4 fits neatly into a `u32`; IPv6, being IPv6, requires a `u128`. Rust's standard library can parse both varieties into an `IpAddr`, and converting the resulting address into the appropriate integer is pleasantly uneventful.

The obvious approach would have been to import all of this into PostgreSQL and query it whenever a request arrived. PostgreSQL range indexes are very good, and this probably would have worked perfectly well for the amount of traffic my personal website receives. It would also mean asking the database the same fundamentally static question on every request, introducing a network hop and turning a lookup over an immutable reference dataset into persistent database work. This offended me aesthetically, so I did something else.

**2) Turning a gigabyte of CSV into something the server actually wants**

I wrote a small Rust preprocessing binary which reads each CSV, parses its address ranges, and inserts them into a `BTreeMap` keyed by the beginning of the range. The map and its entries are encoded using [Bitcode](https://github.com/SoftbearStudios/bitcode), then compressed at zstd level 22 with long-distance matching and multithreading enabled.

This produces two deployment files:

| Database | Original CSV | Processed bundle |
| --- | ---: | ---: |
| IPv4 | 312.5MB | 24.9MB |
| IPv6 | 794.2MB | 42.4MB |

The preprocessing is deliberately done outside the webserver. Parsing more than a gigabyte of CSV and applying an expensive compression level are build/deployment chores, not reasonable things to do whenever the service starts. The resulting bundles are about 67MB combined, which is small enough that I can deploy them alongside the server without feeling particularly guilty.

At startup, the backend decompresses and decodes the IPv4 bundle, converts its raw entries into the runtime representation, drops the temporary decoded allocation, and then repeats the process for IPv6. Doing them sequentially limits the peak memory usage during startup. Location strings are also interned: there are a great many ranges in the files, but comparatively few distinct values such as `United States`, `California`, or `Los Angeles`. Storing one shared copy of each recurring string is considerably less wasteful than keeping millions of identical heap allocations around.

There is probably a still more compact representation involving flat arrays, indexes into dedicated string tables, and memory mapping. That would be faster to load and friendlier to the CPU cache. The current version, however, is already fast enough to make further optimization mostly an exercise in entertaining myself, which admittedly has never stopped me before.

**3) Looking up an address**

The useful property of the source data is that its ranges are ordered and non-overlapping. To find an address, the server:

1. Converts the address to either a `u32` or `u128`.
2. Asks the relevant `BTreeMap` for the last range whose starting address is less than or equal to it.
3. Checks that the address is also less than or equal to that range's ending address.
4. Returns the associated country, region, city, postal code, latitude, and longitude.

In abbreviated Rust, the interesting part is essentially:

```rust
let candidate = map.range(..=address).next_back();

candidate.filter(|(_, entry)| address <= entry.end)
```

This is a predecessor search, giving a lookup complexity of `O(log n)` without scanning the file or consulting another service. The two maps are immutable after startup, so requests can read them concurrently without locks. Geo-IP resolution therefore happens locally, in memory, and without an API key, usage quota, bill, or opportunity for a third party to learn about every visitor to the site.

I exposed this through two small endpoints: one resolves the current client's address, and another accepts an arbitrary IPv4 or IPv6 address. The latter powers a little lookup page on the site and was useful for checking that the data had not been mangled somewhere between CSV parsing and the browser.

It is important to note that Geo-IP is not GPS. A result identifies the location associated with an address range, which may be a city, an ISP office, a corporate gateway, a VPN exit node, or something else entirely. The latitude and longitude should be understood as an approximate marker for the network, not evidence that somebody is sitting in a particular building. This is fortunate, because that would be both unsettling and well beyond the needs of a novelty visitor board.

**4) Recording visits without making every request a database write**

In production, every normal request passes through Axum middleware which extracts the client IP address. The server looks it up in the in-memory Geo-IP maps and discards entries with the database's null-island coordinates of `(0, 0)`. For a valid result, it increments a concurrent in-memory counter keyed by the latitude and longitude. Local, development, and staging requests are excluded, since filling the map with my own test traffic would be both misleading and extremely easy.

The coordinates are converted to their big-endian byte representations before being used as keys. Rust quite sensibly refuses to let floating-point numbers behave as ordinary hash-map keys, since `NaN` makes equality rather philosophical. The source coordinates are fixed values from the database, so their exact bit representations make adequate identities for this purpose.

I initially could have inserted a row into PostgreSQL for every visit, but that would make page requests wait on writes whose results are not immediately important. Instead, visits are accumulated in an `scc::HashMap` and periodically flushed to PostgreSQL in a batch. If acquiring a database connection or inserting the batch fails, the pending counts are merged back into the buffer for a later attempt. The public visitor-board endpoint reads from a separate in-memory aggregate, so loading the map does not require grouping an ever-growing visit table each time.

On startup, the aggregate is reconstructed from the persisted visit records. After that, new visits update it directly. This gives the board persistence across restarts while keeping the hot request path almost entirely in memory.

The current database rows include the visitor IP address, city, country, coordinates, and timestamp because I originally wanted the option of doing traffic analysis later. In retrospect, retaining raw addresses indefinitely is more information than the board needs. A better production version would either avoid persisting the address, truncate or hash it with a rotating salt, aggregate old visits, and establish a clear retention period. Building one's own analytics also means inheriting the unglamorous obligation to decide what should not be collected.

**5) Putting pins on a map**

The front end is written in SolidJS and uses Leaflet to display the aggregate returned by `/api/visitor-board`. Each unique coordinate becomes a marker, and selecting it displays the number of visits associated with that point. Multiple visitors from the same geolocated city collapse naturally into one marker, which both communicates the useful information and prevents the map from becoming an illegible pile of pins.

There is one qualification to my proud declaration of "No external API calls": the Geo-IP resolution and visitor data are completely local, but the current map uses OpenStreetMap's public tile server for the visual basemap. Tiles are static map images rather than a geolocation API, but they are nevertheless external requests. If I decide to become doctrinaire about it, I can host a tile set locally or replace the basemap with a simple bundled world projection. For now, OpenStreetMap receives attribution and saves my miniserver from storing several more gigabytes of cartography.

Leaflet was the fairly obvious choice here. It is mature, small enough, and handles the tedious matters of projections, bounds, zooming, and marker placement. I briefly considered rendering the entire thing myself because apparently I do not value my free time, but learning front-end development does not necessarily require recreating several decades of geographic information systems work.

**6) Was this necessary?**

Absolutely not. A hosted analytics product or a prebuilt visitor-map widget could have provided more features in a small fraction of the time. PostgreSQL alone could also have served the local database without difficulty, and at the scale of this blog nearly any implementation more sophisticated than a paper notebook would cope with the load.

It was nevertheless a useful little project. It involved acquiring and reshaping a real dataset, thinking about compact deployment formats and startup memory use, implementing ordered range lookups, keeping request-time work out of PostgreSQL, and finally making the results visible through a front end. The finished system resolves both IPv4 and IPv6 locally, records visits without blocking requests on individual inserts, and serves an aggregate map without calling a geolocation provider.

More importantly, it looks a little like the web I remember: personal, mildly impractical, and built by somebody who got curious about how a small feature might work and then spent far too long making it from scratch.

Data for this project is provided by [IP2Location LITE](https://lite.ip2location.com).
