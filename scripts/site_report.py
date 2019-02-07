#!/usr/bin/env python3

import statistics
import click
import sys

from pyquery import PyQuery
from collections import namedtuple, defaultdict


Game = namedtuple('Game', ('id', 'type', 'creator', 'players', 'players_by_name'))
Player = namedtuple('Player', ('name', 'version', 'place', 'score'))


@click.command()
@click.option('--profile', default='elsid', help='Player profile')
@click.option('--opponent', default=None, help='Opponent profile (default: all)')
@click.option('--first_page', default=1, help='First pages to fetch', type=int)
@click.option('--last_page', default=1, help='Last pages to fetch', type=int)
@click.option('--first_game_id', default=1, help='First game id to fetch', type=int)
@click.option('--last_game_id', default=sys.maxsize, help='Last game id to fetch', type=int)
@click.option('--version', default=None, help='Version to check (default: all)', type=int)
@click.option('--sort_by', default='n', help='Sort by field')
def main(profile, opponent, first_page, last_page, first_game_id, last_game_id, version, sort_by):
    games = list(fetch_games(profile, first_page, last_page))
    show_report(games, profile, version, opponent, first_game_id, last_game_id, sort_by)


def show_report(games, profile, version, opponent, first_game_id, last_game_id, sort_by):
    games = [v for v in games if check_game(v, profile, version, opponent, first_game_id, last_game_id)]
    show_stats_by_game_type(games, profile, sort_by)
    show_stats_by_opponent(games, profile, sort_by)
    show_stats_by_opponent_and_version(games, profile, sort_by)
    show_stats_by_opponent_and_game_type(games, profile, sort_by)
    show_stats_by_opponent_and_version_and_game_type(games, profile, sort_by)


def check_game(game, profile, version, opponent, first_game_id, last_game_id):
    return (
            profile in game.players_by_name
            and (opponent is None or opponent in game.players_by_name)
            and (version is None or game.players_by_name[profile].version == version)
            and first_game_id <= game.id <= last_game_id
    )


def show_stats_by_game_type(games, profile, sort_by):
    stats = defaultdict(lambda: dict(scores=list(), places=list()))
    for game in games:
        stats[game.type]['scores'].append(game.players_by_name[profile].score)
        stats[game.type]['places'].append(game.players_by_name[profile].place)
    print()
    row('game_type', 'n', 'total_score', 'mean_score', 'median_score', 'mean_place', 'median_place')
    show_stats(stats, sort_by)


def show_stats_by_opponent(games, profile, sort_by):
    stats = defaultdict(lambda: dict(scores=list(), places=list()))
    for game in games:
        opponent = next(v for v in game.players_by_name.keys() if v != profile)
        stats[opponent]['scores'].append(game.players_by_name[profile].score)
        stats[opponent]['places'].append(game.players_by_name[profile].place)
    print()
    row('opponent', 'n', 'total_score', 'mean_score', 'median_score', 'mean_place', 'median_place')
    show_stats(stats, sort_by)


def show_stats_by_opponent_and_version(games, profile, sort_by):
    stats = defaultdict(lambda: dict(scores=list(), places=list()))
    for game in games:
        opponent = next(v for v in game.players_by_name.keys() if v != profile)
        key = (opponent, game.players_by_name[opponent].version)
        stats[key]['scores'].append(game.players_by_name[profile].score)
        stats[key]['places'].append(game.players_by_name[profile].place)
    print()
    row('opponent', 'version', 'n', 'total_score', 'mean_score', 'median_score', 'mean_place', 'median_place')
    show_stats(stats, sort_by)


def show_stats_by_opponent_and_game_type(games, profile, sort_by):
    stats = defaultdict(lambda: dict(scores=list(), places=list()))
    for game in games:
        opponent = next(v for v in game.players_by_name.keys() if v != profile)
        key = (opponent, game.type)
        stats[key]['scores'].append(game.players_by_name[profile].score)
        stats[key]['places'].append(game.players_by_name[profile].place)
    print()
    row('opponent', 'game_type', 'n', 'total_score', 'mean_score', 'median_score', 'mean_place', 'median_place')
    show_stats(stats, sort_by)


def show_stats_by_opponent_and_version_and_game_type(games, profile, sort_by):
    stats = defaultdict(lambda: dict(scores=list(), places=list()))
    for game in games:
        opponent = next(v for v in game.players_by_name.keys() if v != profile)
        key = (opponent, game.players_by_name[opponent].version, game.type)
        stats[key]['scores'].append(game.players_by_name[profile].score)
        stats[key]['places'].append(game.players_by_name[profile].place)
    print()
    row('opponent', 'version', 'game_type', 'n', 'total_score', 'mean_score', 'median_score', 'mean_place', 'median_place')
    show_stats(stats, sort_by)


def show_stats(stats, sort_by):
    rows = sorted((make_row(k, v) for k, v in stats.items()), key=lambda v: v[sort_by])
    total = dict(scores=list(), places=list())
    for v in stats.values():
        total['scores'] += v['scores']
        total['places'] += v['places']
    for v in rows:
        show_row(v)
    show_row(make_row('total', total))


def make_rows(stats):
    total = dict(scores=list(), places=list())
    rows = list()
    for key, values in stats.items():
        total['scores'] += values['scores']
        total['places'] += values['places']
        rows.append(make_row(key, values))


def make_row(key, values):
    n = len(values['scores'])
    return dict(
        key=key,
        n=n,
        total_score=sum(values['scores']),
        mean_score=statistics.mean(values['scores']) if n > 0 else 0,
        median_score=statistics.median(values['scores']) if n > 0 else 0,
        mean_place=statistics.mean(values['places']) if n > 0 else 0,
        median_place=statistics.median(values['places']) if n > 0 else 0,
    )


def show_row(values):
    key = [values['key']] if isinstance(values['key'], str) else values['key']
    del values['key']
    row(*key, *list(values.values()))


def fetch_games(profile, first_page, last_page):
    for page in range(first_page, last_page + 1):
        root = PyQuery(url=f'http://russianaicup.ru/profile/{profile}/allGames/page/{page}')
        rows = PyQuery(root('.gamesTable > tbody:nth-child(3)').html())
        yield from (v for v in rows('tr').map(lambda k, v: parse_game(PyQuery(v))) if v)


def parse_game(query):
    if query('td:nth-child(7)').text() == 'Game is testing now':
        return None
    players = parse_players(query)
    return Game(
        id=int(query('td:nth-child(1) > a:nth-child(1)').text()),
        type=query('td:nth-child(2)').text().replace('\u00d7', 'x'),
        creator=query('td:nth-child(4) > div:nth-child(1)').text(),
        players=players,
        players_by_name={v.name: v for v in players},
    )


def parse_players(query):
    return [parse_first_player(query), parse_second_player(query)]


def parse_first_player(query):
    return Player(
        name=get_player_name(query('td:nth-child(5) > a:nth-child(1)')),
        version=int(query('td:nth-child(6)').text().split('\n')[0]),
        score=int(query('td:nth-child(7) > div:nth-child(1)').text()),
        place=int(query('td:nth-child(8) > div:nth-child(1)').text()),
    )


def parse_second_player(query):
    return Player(
        name=get_player_name(query('td:nth-child(5) > a:nth-child(4)')),
        version=int(query('td:nth-child(6)').text().split('\n')[1]),
        score=int(query('td:nth-child(7) > div:nth-child(3)').text()),
        place=int(query('td:nth-child(8) > div:nth-child(3)').text()),
    )


def get_player_name(query):
    return query.attr('href').split('/')[-1]


def row(*args):
    print(('{:>20}' * len(args)).format(*args))


if __name__ == '__main__':
    main()
