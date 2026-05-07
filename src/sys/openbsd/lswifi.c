/*
 * Copyright (c) 2018 Andreas Koerner <andi@jaak.de>
 * Copyright (c) 2026 simon0302010 <simon@hackclub.app>
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */


#include <errno.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <sys/time.h>
#include <sys/queue.h>

#include <ifaddrs.h>
#include <net/if.h>
#include <net/route.h>
#include <netinet/in.h>
#include <netinet/if_ether.h>
#include <netinet6/in6_var.h>

#include <net80211/ieee80211.h>
#include <net80211/ieee80211_ioctl.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>
#include <err.h>

static int wifi_interfaces = 0;

/* ---------------------------------------------------------------------- */

/* Structures
   ---------- */


/* all relevant data for a single wifi network */
SLIST_HEAD(wifidat_head, wifidat) interfaces;
struct wifidat {
	SLIST_ENTRY(wifidat) elems;
	const char* interface;            /* interface name */
	struct ieee80211_nodereq_all na;  /* result for ioctl(SCAN) */
	struct ieee80211_nodereq nr[512]; /* list of all networks */

	struct ieee80211_nwid nwid;       /* connected network name */
	struct ieee80211_bssid bssid;     /* connected netword id */
};

typedef struct {
	const char *interface;
	int connected;
	char *ssid;
	char *bssid;
	int rssi;
	int channel;
	uint nr_capinfo;
	uint nr_rsnprotos;
	uint nr_rsnakms;
} lswifi_result;

int network_name_is_sane(const u_char* name, int len)
{
	if (len > IEEE80211_NWID_LEN)
		return 0;

	for (int i = 0; i < len; i++)
		if (name[i] & 0x80 || !isprint(name[i]))
			return 0;

	return 1;
}

void format_interface_data(struct wifidat* data, lswifi_result **networks, int *networks_idx)
{
	int i, len, connected;
	struct ieee80211_nodereq* network;

	for (i = 0; i < data->na.na_nodes; i++) {
		network = &data->nr[i];

		/* Network name (not zero ended, sadly) */
		len = network->nr_nwid_len;
		if (!network_name_is_sane(network->nr_nwid, len))
			continue;

		/* connected to that network? */
		connected = (len == data->nwid.i_len
			&& memcmp(network->nr_nwid, data->nwid.i_nwid, len) == 0
			&& memcmp(network->nr_bssid, data->bssid.i_bssid, IEEE80211_ADDR_LEN) == 0
		);

		char *bssid = strdup(ether_ntoa((struct ether_addr *)network->nr_bssid));

		int rssi = network->nr_max_rssi
			? -IEEE80211_NODEREQ_RSSI(network)
			: network->nr_rssi;

		char *ssid;
		if (asprintf(&ssid, "%.*s", len, network->nr_nwid) == -1) {
			perror("asprintf");
			free(bssid);
			continue;
		}

		// printf("%s: nr_capinfo = %u; nr_rsnprotos = %u; nr_rsnakms = %u\n", ssid, network->nr_capinfo, network->nr_rsnprotos, network->nr_rsnakms);

		lswifi_result *result = malloc(sizeof(lswifi_result));
		if (result == NULL) {
			free(ssid);
			free(bssid);
		} else {
			*result = (lswifi_result){
				.interface = strdup(data->interface),
				.connected = connected,
				.ssid = ssid,
				.bssid = bssid,
				.rssi = rssi,
				.channel = network->nr_channel,
				.nr_capinfo = network->nr_capinfo & ~IEEE80211_CAPINFO_ESS,
				.nr_rsnprotos = network->nr_rsnprotos,
				.nr_rsnakms = network->nr_rsnakms
			};

			networks[*networks_idx] = result;
			(*networks_idx)++;
		}
	}
}

int query_interface(const char* if_name, struct wifidat* data)
{
	int sock;
	struct ifreq ifr;     /* request - input to/output from ioctl */
	int inwid, ibssid;

	sock = socket(AF_INET, SOCK_DGRAM, 0);
	if (sock < 0) {
		perror("socket");
		return -1;
	}

	/* store interface name for later */
	data->interface = if_name;

	/* check if valid and connected wifi network */
	
	bzero(&ifr, sizeof(ifr));
	ifr.ifr_data = (caddr_t)&data->nwid;
	strlcpy(ifr.ifr_name, if_name, sizeof(ifr.ifr_name));
	inwid = ioctl(sock, SIOCG80211NWID, (caddr_t)&ifr);

	bzero(&data->bssid, sizeof(data->bssid));
	strlcpy(data->bssid.i_name, if_name, sizeof(data->bssid.i_name));
	ibssid = ioctl(sock, SIOCG80211BSSID, &data->bssid);

	/* check if any ieee80211 option is active */
	if (inwid != 0 && ibssid != 0) {
		close(sock);
		return -1;
	}

	/* copy the name of the interface to the request struct */
	bzero(&ifr, sizeof(ifr));
	strlcpy(ifr.ifr_name, if_name, sizeof(ifr.ifr_name));
	
	/* scan or, if error, return (the interface is no wifi...)*/
	if (ioctl(sock, SIOCS80211SCAN, (caddr_t)&ifr) != 0) {
		perror("ioctl");
		printf("code %i", errno);
		close(sock);
		return -1;
	}

	/* get the scan result */
	bzero(&data->na, sizeof(data->na));
	bzero(&data->nr, sizeof(data->nr));
	data->na.na_node = data->nr;
	data->na.na_size = sizeof(data->nr);
	strlcpy(data->na.na_ifname, if_name, sizeof(data->na.na_ifname));

	if (ioctl(sock, SIOCG80211ALLNODES, &data->na) != 0) {
		warn("SIOCG80211ALLNODES");
		close(sock);
		return -1;
	}

	if (!data->na.na_nodes) {
		fprintf(stderr, "warning: no networks found on %s\n", if_name);
	}

	close(sock);

	wifi_interfaces++;

	return 0;
}

void free_networks(lswifi_result **networks)
{
	if (!networks) return;
	for (int i = 0; networks[i] != NULL; i++) {
		free(networks[i]->bssid);
		free(networks[i]->ssid);
		free(networks[i]->interface);
		free(networks[i]);
	}
	free(networks);
}

lswifi_result **get_networks()
{
	struct ifaddrs *ifap; /* all existing interfaces */
	struct ifaddrs *ifa;  /* current interface in iteration over ifap */
	struct wifidat* data;

	SLIST_INIT(&interfaces);
	
	/* iterate over interfaces */
	if (getifaddrs(&ifap) != 0) {
		perror("getifaddrs");
		return NULL;
	}

	wifi_interfaces = 0;

	/* query for interfaces and networks */
	for (ifa = ifap; ifa; ifa = ifa->ifa_next) {
		data = malloc(sizeof(struct wifidat));

		if (query_interface(ifa->ifa_name, data) == 0) {
			SLIST_INSERT_HEAD(&interfaces, data, elems);
		} else {
			free(data);
			continue;
		}
	}

    if (wifi_interfaces == 0) {
        errno = ENXIO;
        goto on_fail;
    }

	int total_networks = 0;
	SLIST_FOREACH(data, &interfaces, elems)
		total_networks += data->na.na_nodes;

	lswifi_result **networks = malloc(sizeof(lswifi_result *) * (total_networks + 1));
	if (!networks)
		goto on_fail;

	int networks_idx = 0;

	/* put the the result of the scan into the struct */
	SLIST_FOREACH(data, &interfaces, elems)
		format_interface_data(data, networks, &networks_idx);

	networks[networks_idx] = NULL;

	freeifaddrs(ifap);

	while (!SLIST_EMPTY(&interfaces)) {
		data = SLIST_FIRST(&interfaces);
		SLIST_REMOVE_HEAD(&interfaces, elems);
		free(data);
	}

	return networks;

on_fail:
	freeifaddrs(ifap);

	while (!SLIST_EMPTY(&interfaces)) {
		data = SLIST_FIRST(&interfaces);
		SLIST_REMOVE_HEAD(&interfaces, elems);
		free(data);
	}

	free_networks(networks);

	return NULL;
}