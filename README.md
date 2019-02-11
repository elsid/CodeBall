# RAIC 2018 - CodeBall bot

Bot implementation by http://2018.russianaicup.ru/profile/elsid .

![render](https://imgur.com/91XKJn2.jpg)

## Usage

Build and run using local runner vs helper:
```bash
scripts/run_vs_helper.sh
```

## Simulation tool

Based on strategy simulation implementation.
Now supports fast simulation tests to check goalkeeper quality (percent of safes).
Allows to generate situations with random ball position and velocity.
Uses this situations to check goalkeeper strategy can it safe goal from ball in 150 game ticks.
Simulations contains only arena, ball and goalkeeper. No other robots are included.

Usage:
1. Build simulation tool:
```bash
cargo build --features disable_output,use_single_goalkeeper --release --bin simulation_tool
```
2. Generate goal situations:
```bash
target/release/simulation_tool generate_empty 10000 > empty.json
```
3. Filter hits (empty.json contains all results with goals and without):
```bash
scripts/filter_simulations.py hit < empty.json > hits.json
```
4. Check goalkeeper:
```bash
target/release/simulation_tool check_goalkeeper < hits.json > goalkeeper.json
```
5. Join goakeeper results with empty (if need to have full statistics):
```bash
scripts/join_simulations.py goalkeeper.json empty.json > empty_and_goalkeeper.json
```
6. See [check goalkeeper report](check-goalkeeper-report) script how to get visual results.

## Scripts

### Stats report

[scripts/stats_report.py](scripts/stats_report.py) produces report with graphics and tables base on `enable_stats` features.

Usage:

1. Build strategy with `--features=enable_stats`
```bash
cargo build --features=enable_stats
```
2. Run it using local runner:
```bash
target/release/my-strategy 127.0.0.1 ${PORT} 0000000000000000 > stats.json
```
3. Build report:
```bash
scripts/stats_report.py < stats.json
```

Example:

```
scripts/stats_report.py < log/stats.1549829457.json
                                                 n                      sum                      q95                      min                      max                     mean                   median                    stdev
                player_id                    40879                    74975                      2.0                        1                        2        1.834071283544118                        2       0.3720211863681954
                 robot_id                    40879                   130200                      4.0                        1                        4        3.185009418038602                        3       0.8643186159515917
             current_tick                    40879                366519610                  17157.0                        0                    17999        8965.963208493358                     9043       5290.7518385241665
                    order                        3                        -                        -                idle (80)             play (31131)
             time_to_jump                    40879        10654.26666666669       1.1333333333333342                      0.0       1.6666666666666656      0.26062933698639085                      0.0       0.4064722516234682
            time_to_watch                    40879       12389.666666666726       1.2333333333333338                      0.0       1.6666666666666656      0.30308145176414963      0.03333333333333333       0.4244621053586586
              time_to_end                    40879        49303.51666665346       1.6666666666666656                      0.0       1.6666666666666656       1.2060842160196346       1.6666666666666656       0.7120357255180221
            time_to_score                     4584        5071.366666666668       1.6333333333333324     0.016666666666666666       1.6666666666666656       1.1063190808609658       1.2000000000000006      0.43295545202682695
                iteration                    40879                  3758551                    439.0                        0                     4914        91.94332053132415                        2       224.15174096247264
         total_iterations                    40879                 12305431                   1264.0                        0                     6859       301.02084199711345                        5        581.3825766074057
               game_score                    40879                     1520                      1.0                       -1                        1      0.03718290564837692                        0      0.33280017479505514
              order_score                    40879                 30911349                   2634.0                    -1234                     3203        756.1669561388488                      778        804.3125921954689
         path_micro_ticks                    40879                 24264343                   2500.0                        0                     2800        593.5649844663519                      300        785.8256499248528
         plan_micro_ticks                    40879               1096339133       104847.19999999995                        0                   519391       26819.127987475233                     3328        50746.73682799437
         game_micro_ticks                    40879           17244133289278             1044972509.4                        0               1120780908        421833540.1863549                381218969       367894391.70325005
   game_micro_ticks_limit                    40879         2825881150000000       169830999999.99997                        0             180010000000        69127942219.72162              61460000000        60642987790.03748
             current_step                    40879                    17696                      1.0                        0                        1       0.4328873015484723                        0       0.4954814742059794
       reached_game_limit                    40879                        0                      0.0                        0                        0                        0                        0                      0.0
       reached_plan_limit                    40879                       83                      0.0                        0                        1     0.002030382347904792                        0      0.04501454724747978
       reached_path_limit                    40879                        0                      0.0                        0                        0                        0                        0                      0.0
             other_number                    40879                    17459                      2.0                        0                        2       0.4270897037598767                        0        0.628638746861726
ticks_with_near_micro_ticks                    40879                  1534988                    100.0                        0                      116        37.54954866802026                       26        36.70976257562621
ticks_with_far_micro_ticks                    40879                  2265277                    100.0                        0                      120       55.414197998972575                       81       46.094717172448505
                     path                       26                        -                        -fork_ball,walk_to_ball,push_ball,push_ball,jump,watch_me_jump,watch_ball_move (1)fork_ball,walk_to_position,jump,watch_me_jump,watch_ball_move (11881)
              transitions                       12                        -                        -           push_robot (1)    watch_me_jump (29080)
```

![graphics](https://imgur.com/2IN4yOq.jpg)

### Site report

[scripts/site_report.py](scripts/site_report.py) shows statistics report based on site games.

Usage:

```
scripts/site_report.py --help
Usage: site_report.py [OPTIONS]

Options:
  --profile TEXT           Player profile
  --opponent TEXT          Opponent profile (default: all)
  --first_page INTEGER     First pages to fetch
  --last_page INTEGER      Last pages to fetch
  --first_game_id INTEGER  First game id to fetch
  --last_game_id INTEGER   Last game id to fetch
  --version INTEGER        Version to check (default: all)
  --sort_by TEXT           Sort by field
  --creator TEXT           Game creator (default: all)
  --help                   Show this message and exit.
```

Example:

```
scripts/site_report.py --first_page=1 --last_page=3 --sort_by=mean_place --creator=System

           game_type                   n         total_score          mean_score        median_score          mean_place        median_place
                2x3+                  15                 129                 8.6                   7                 1.4                   1
                2x2+                   6                  25   4.166666666666667                 3.5                 1.5                 1.5
                 2x2                   9                  57   6.333333333333333                   6  1.5555555555555556                   2
               total                  30                 211   7.033333333333333                 5.5  1.4666666666666666                 1.0

            opponent                   n         total_score          mean_score        median_score          mean_place        median_place
            robostac                   1                  16                  16                  16                   1                   1
             ykaland                   1                   5                   5                   5                   1                   1
          sergio-dna                   1                  12                  12                  12                   1                   1
              MucmuK                   1                  13                  13                  13                   1                   1
          FunnyHouse                   1                  23                  23                  23                   1                   1
               dedoo                   2                  16                   8                 8.0                   1                 1.0
                iam1                   1                  18                  18                  18                   1                   1
               RiSuS                   1                   7                   7                   7                   1                   1
          discourage                   1                  11                  11                  11                   1                   1
       concretemixer                   1                   9                   9                   9                   1                   1
                 cas                   5                  32                 6.4                   8                 1.4                   1
              mixei4                   2                  10                   5                 5.0                 1.5                 1.5
                lama                   3                  12                   4                   4  1.6666666666666667                   2
           Enchante_                   1                   3                   3                   3                   2                   2
            Daramant                   2                   3                 1.5                 1.5                   2                 2.0
              Antmsu                   1                   4                   4                   4                   2                   2
           giperball                   3                   7  2.3333333333333335                   2                   2                   2
                 Lev                   1                   2                   2                   2                   2                   2
         evil_homura                   1                   8                   8                   8                   2                   2
               total                  30                 211   7.033333333333333                 5.5  1.4666666666666666                 1.0

            opponent             version                   n         total_score          mean_score        median_score          mean_place        median_place
            robostac                 127                   1                  16                  16                  16                   1                   1
             ykaland                  28                   1                   5                   5                   5                   1                   1
          sergio-dna                  33                   1                  12                  12                  12                   1                   1
              MucmuK                  17                   1                  13                  13                  13                   1                   1
          FunnyHouse                  37                   1                  23                  23                  23                   1                   1
               dedoo                  15                   2                  16                   8                 8.0                   1                 1.0
                iam1                  26                   1                  18                  18                  18                   1                   1
               RiSuS                  63                   1                   7                   7                   7                   1                   1
          discourage                  54                   1                  11                  11                  11                   1                   1
       concretemixer                  51                   1                   9                   9                   9                   1                   1
                 cas                  26                   5                  32                 6.4                   8                 1.4                   1
              mixei4                  27                   2                  10                   5                 5.0                 1.5                 1.5
                lama                  22                   3                  12                   4                   4  1.6666666666666667                   2
           Enchante_                  20                   1                   3                   3                   3                   2                   2
            Daramant                  21                   2                   3                 1.5                 1.5                   2                 2.0
              Antmsu                  54                   1                   4                   4                   4                   2                   2
           giperball                  88                   3                   7  2.3333333333333335                   2                   2                   2
                 Lev                  23                   1                   2                   2                   2                   2                   2
         evil_homura                  29                   1                   8                   8                   8                   2                   2
               total                  30                 211   7.033333333333333                 5.5  1.4666666666666666                 1.0

            opponent           game_type                   n         total_score          mean_score        median_score          mean_place        median_place
            robostac                2x3+                   1                  16                  16                  16                   1                   1
             ykaland                2x2+                   1                   5                   5                   5                   1                   1
          sergio-dna                 2x2                   1                  12                  12                  12                   1                   1
              MucmuK                2x3+                   1                  13                  13                  13                   1                   1
          FunnyHouse                2x3+                   1                  23                  23                  23                   1                   1
               dedoo                2x3+                   1                   7                   7                   7                   1                   1
                iam1                2x3+                   1                  18                  18                  18                   1                   1
               RiSuS                2x3+                   1                   7                   7                   7                   1                   1
                 cas                 2x2                   1                   8                   8                   8                   1                   1
                lama                2x2+                   1                   4                   4                   4                   1                   1
               dedoo                2x2+                   1                   9                   9                   9                   1                   1
          discourage                 2x2                   1                  11                  11                  11                   1                   1
       concretemixer                 2x2                   1                   9                   9                   9                   1                   1
                 cas                2x3+                   4                  24                   6                 7.0                 1.5                 1.5
              mixei4                2x3+                   2                  10                   5                 5.0                 1.5                 1.5
           Enchante_                 2x2                   1                   3                   3                   3                   2                   2
                lama                 2x2                   2                   8                   4                 4.0                   2                 2.0
            Daramant                2x3+                   1                   1                   1                   1                   2                   2
              Antmsu                 2x2                   1                   4                   4                   4                   2                   2
            Daramant                2x2+                   1                   2                   2                   2                   2                   2
           giperball                2x2+                   2                   5                 2.5                 2.5                   2                 2.0
                 Lev                2x3+                   1                   2                   2                   2                   2                   2
           giperball                 2x2                   1                   2                   2                   2                   2                   2
         evil_homura                2x3+                   1                   8                   8                   8                   2                   2
               total                  30                 211   7.033333333333333                 5.5  1.4666666666666666                 1.0

            opponent             version           game_type                   n         total_score          mean_score        median_score          mean_place        median_place
            robostac                 127                2x3+                   1                  16                  16                  16                   1                   1
             ykaland                  28                2x2+                   1                   5                   5                   5                   1                   1
          sergio-dna                  33                 2x2                   1                  12                  12                  12                   1                   1
              MucmuK                  17                2x3+                   1                  13                  13                  13                   1                   1
          FunnyHouse                  37                2x3+                   1                  23                  23                  23                   1                   1
               dedoo                  15                2x3+                   1                   7                   7                   7                   1                   1
                iam1                  26                2x3+                   1                  18                  18                  18                   1                   1
               RiSuS                  63                2x3+                   1                   7                   7                   7                   1                   1
                 cas                  26                 2x2                   1                   8                   8                   8                   1                   1
                lama                  22                2x2+                   1                   4                   4                   4                   1                   1
               dedoo                  15                2x2+                   1                   9                   9                   9                   1                   1
          discourage                  54                 2x2                   1                  11                  11                  11                   1                   1
       concretemixer                  51                 2x2                   1                   9                   9                   9                   1                   1
                 cas                  26                2x3+                   4                  24                   6                 7.0                 1.5                 1.5
              mixei4                  27                2x3+                   2                  10                   5                 5.0                 1.5                 1.5
           Enchante_                  20                 2x2                   1                   3                   3                   3                   2                   2
                lama                  22                 2x2                   2                   8                   4                 4.0                   2                 2.0
            Daramant                  21                2x3+                   1                   1                   1                   1                   2                   2
              Antmsu                  54                 2x2                   1                   4                   4                   4                   2                   2
            Daramant                  21                2x2+                   1                   2                   2                   2                   2                   2
           giperball                  88                2x2+                   2                   5                 2.5                 2.5                   2                 2.0
                 Lev                  23                2x3+                   1                   2                   2                   2                   2                   2
           giperball                  88                 2x2                   1                   2                   2                   2                   2                   2
         evil_homura                  29                2x3+                   1                   8                   8                   8                   2                   2
               total                  30                 211   7.033333333333333                 5.5  1.4666666666666666                 1.0
```

### Results stats

[scripts/results_stats.py](scripts/results_stats.py) produces graphics and table report for multiple `result.txt` files from local runner launch.

Usage:

1. Run local runner with `--results-file` option multiple times using different file names.
2. Run report script:
```bash
scripts/results_stats.py result.*.txt
```

Example:

```
scripts/results_stats.py log/v29_vs_a0ef50a/*
                    games                       50
             unique seeds                       50

                                             first                   second     ratio (second/first)
                       _1                       14                       41       2.9285714285714284
                       _2                       36                        9                     0.25
              total_score                      172                      286       1.6627906976744187
                min_score                        1                        2                      2.0
                max_score                        8                       12                      1.5
               mean_score                     3.44                     5.72       1.6627906976744184
             median_score                      3.0                      5.0       1.6666666666666667
              stdev_score       1.9708073555406327       2.4498229840258303        1.243055531094156
                q95_score                      7.0                     10.0       1.4285714285714286
           min_score_diff                      -10                       -4                      0.4
           max_score_diff                        4                       10                      2.5
          mean_score_diff                    -2.28                     2.28                     -1.0
        median_score_diff                     -2.0                      2.0                     -1.0
         stdev_score_diff       3.2390474790327746       3.2390474790327746                        1

1908789847 2128538004 432241528 730002022 283945802 630688912 2607963432 3714742159 1670516122 2647676947 2046736159 2974637113 508816090 2564744498 4089256807 1229238542 2004587353 561612246 901130959 4227012945 3231268869 3770445239 2186040580 3888576320 988935913 3600086806 3846219263 1597635489 1718150755 283133820 3029431657 2525454198 322098379 3588740731 4178094541 2288098860 1472899594 1486267384 1329421881 113597890 3674966996 4049527531 1451810575 1909916721 520566021 4223512919 1537191805 776895816 3568852997 2618877841
```

![graphics](https://imgur.com/1Tr29h1.jpg)

### Game report

[scripts/game_report.py](scripts/game_report.py) produces graphics and table report for local runner game log.

Usage:

1. Run local runner with `--log-file` option.
2. Run report script:
```bash
scripts/game_report.py < game.log
```

Example:
```
scripts/game_report.py < 389180.c1_scauto4z5zwmzazyszyjcvmpapxte.log
                                 elsid-4             elsid-5             elsid-6            mixei4-1            mixei4-2            mixei4-3               elsid              mixei4               ratio
          mean speed   19.48112740336314  17.352230030805032   19.67599215415863  17.051426009221405  17.851066404799653   16.47375186164014  18.836449862775602   17.12541475855373   1.099912038823308
            distance   90.96201222502278    80.9972738930335   91.85308225919583   79.28460161369888    83.1239941638729   76.76884941696686   263.8123683772521  239.17744519453862  1.1029985212973417
         mean radius   1.004345369103563    1.00334304919398  1.0036791386592112  1.0013899232645291  1.0011974897388614  1.0002855273332936  1.0037891856522516  1.0009576467788948  1.0028288298535595
       median radius   1.004345369103563    1.00334304919398  1.0036791386592112  1.0013899232645291  1.0011974897388614  1.0002855273332936  1.0037891856522516  1.0009576467788948  1.0028288298535595
                hits                   -                   -                   -                   -                   -                   -                 162                 115  1.4086956521739131
      initial ball.y               ticks              winner
   5.949698463075475                 598              mixei4
   6.614135030758736                5082               elsid
   6.890847866625197                 624              mixei4
   6.045366164172275                2078               elsid
   2.554480156112114                 617               elsid
    4.76723537014998                1738              mixei4
  4.9140376245386035                 898              mixei4
  7.7303627052045485                 958              mixei4
  2.4078679586678198                 452               elsid
  3.5857436392422053                2759               elsid
```

![graphics](https://imgur.com/F5RoWVu.jpg)

### Check goalkeeper report

[scripts/check_goalkeeper_report.py](scripts/check_goalkeeper_report.py) produces graphics and table report for simulation tool check goalkeeper result.

Usage:

1. See [simulation tool](simulation-tool) how to prepare data.
2. Run report script:
```bash
scripts/check_goalkeeper_report.py < empty_and_goalkeeper.json
```

Example:
```
scripts/check_goalkeeper_report.py < log/empty_and_goalkeeper.1549269720.1549916017.c64e615.10000.json
                source           simulations                misses                  hits             hits rate                 safes                 goals            goals rate            safes rate           safes/goals
                 stdin                 10000                  8616                  1384    0.1606313834726091                  1170                   214    0.1546242774566474    0.8453757225433526    5.4672897196261685
```

![graphics](https://imgur.com/RNkqitD.jpg)
