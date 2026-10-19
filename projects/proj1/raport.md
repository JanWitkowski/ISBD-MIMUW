## Latency
Testy są wykonywane na 256 MiB pliku.

### Badanie czasu wykonania programu
Wyniki pierwszej wersji która po odczytaniu każdego bloku przesyła go do computa oraz zmiennej synchronicznej która blokuje dowiadywanie się jaki kolejny indeks offsetu dobrać.

| Rozmiar (M) | Wątki (threads) | Czas minimalny (min) |
| --- | --- | --- |
| M=512 B | threads=1 | min=703201 µs |
| M=512 B | threads=2 | min=4576691 µs |
| M=512 B | threads=4 | min=5587110 µs |
| M=512 B | threads=8 | min=8264896 µs |
| M=512 B | threads=16 | min=9467424 µs |
| M=4096 B | threads=1 | min=864513 µs |
| M=4096 B | threads=2 | min=1739020 µs |
| M=4096 B | threads=4 | min=1899349 µs |
| M=4096 B | threads=8 | min=2121323 µs |
| M=4096 B | threads=16 | min=2073248 µs |
| M=16384 B | threads=1 | min=627797 µs |
| M=16384 B | threads=2 | min=677632 µs |
| M=16384 B | threads=4 | min=693754 µs |
| M=16384 B | threads=8 | min=649146 µs |
| M=16384 B | threads=16 | min=656664 µs |
| M=65536 B | threads=1 | min=473788 µs |
| M=65536 B | threads=2 | min=473623 µs |
| M=65536 B | threads=4 | min=480105 µs |
| M=65536 B | threads=8 | min=490356 µs |
| M=65536 B | threads=16 | min=487035 µs |
| M=1048576 B | threads=1 | min=411011 µs |
| M=1048576 B | threads=2 | min=412162 µs |
| M=1048576 B | threads=4 | min=419717 µs |
| M=1048576 B | threads=8 | min=420484 µs |
| M=1048576 B | threads=16 | min=431537 µs |

W skrócie narzut synchronizacji za duży. Powstała więc wersja dwa, która poprawia poprzednie błędy, implementując batchowanie i dostawanie przez readery wycinku tablicy z wylosowanymi offsetami. Wyniki teraz wyglądają juz lepiej nie pogarszając przynajmniej bardzo z liczbą dodawania wątków czasu. Jednak przez brak czyszczenia cachea nie ma poprawy przy dodawaniu wątków.

| Rozmiar (M) | Wątki (threads) | Czas minimalny (min) |
| --- | --- | --- |
| M=512 B | threads=1 | min=479108 µs |
| M=512 B | threads=2 | min=473862 µs |
| M=512 B | threads=4 | min=476733 µs |
| M=512 B | threads=8 | min=482312 µs |
| M=512 B | threads=16 | min=502158 µs |
| M=4096 B | threads=1 | min=435667 µs |
| M=4096 B | threads=2 | min=440600 µs |
| M=4096 B | threads=4 | min=441396 µs |
| M=4096 B | threads=8 | min=439759 µs |
| M=4096 B | threads=16 | min=474585 µs |
| M=16384 B | threads=1 | min=436782 µs |
| M=16384 B | threads=2 | min=433111 µs |
| M=16384 B | threads=4 | min=434282 µs |
| M=16384 B | threads=8 | min=446231 µs |
| M=16384 B | threads=16 | min=450578 µs |
| M=65536 B | threads=1 | min=436464 µs |
| M=65536 B | threads=2 | min=428329 µs |
| M=65536 B | threads=4 | min=425722 µs |
| M=65536 B | threads=8 | min=431999 µs |
| M=65536 B | threads=16 | min=437073 µs |
| M=1048576 B | threads=1 | min=414642 µs |
| M=1048576 B | threads=2 | min=408683 µs |
| M=1048576 B | threads=4 | min=418046 µs |
| M=1048576 B | threads=8 | min=444110 µs |
| M=1048576 B | threads=16 | min=435599 µs |

Już z działającym programem poniższe wyniki uzyskane przy każdorazowym czyszczeniu cache pokazują wyraźne zmniejszenie czasu wykonania programu z dodawaniem kolejnych wątków które jednak wraz ze wzrostem wielkości bloków zwalnia a nawet już przy blokach wielkości pół i jednego megabytea używanie 8 i 16 wątków pogarsza dla 256 megabytowego pliku, co ma sens jako że wtedy jeden wątek jest tworzony tylko do paru odczytów, więc narzut synchroniczny jest bardziej widoczny. 

| Rozmiar (M) | Wątki (threads) | Czas minimalny (min) |
| --- | --- | --- |
| M=512 B | threads=1 | cold-cache min=6338167 µs |
| M=512 B | threads=2 | cold-cache min=3280668 µs |
| M=512 B | threads=4 | cold-cache min=2024113 µs |
| M=512 B | threads=8 | cold-cache min=1383672 µs |
| M=512 B | threads=16 | cold-cache min=1121312 µs |
| M=4096 B | threads=1 | cold-cache min=5795053 µs |
| M=4096 B | threads=2 | cold-cache min=3408967 µs |
| M=4096 B | threads=4 | cold-cache min=2036817 µs |
| M=4096 B | threads=8 | cold-cache min=1227586 µs |
| M=4096 B | threads=16 | cold-cache min=1145780 µs |
| M=16384 B | threads=1 | cold-cache min=2318101 µs |
| M=16384 B | threads=2 | cold-cache min=1285115 µs |
| M=16384 B | threads=4 | cold-cache min=883710 µs |
| M=16384 B | threads=8 | cold-cache min=652439 µs |
| M=16384 B | threads=16 | cold-cache min=698239 µs |
| M=65536 B | threads=1 | cold-cache min=903380 µs |
| M=65536 B | threads=2 | cold-cache min=638474 µs |
| M=65536 B | threads=4 | cold-cache min=568152 µs |
| M=65536 B | threads=8 | cold-cache min=579912 µs |
| M=65536 B | threads=16 | cold-cache min=648374 µs |
| M=1048576 B | threads=1 | cold-cache min=508966 µs |
| M=1048576 B | threads=2 | cold-cache min=514384 µs |
| M=1048576 B | threads=4 | cold-cache min=506798 µs |
| M=1048576 B | threads=8 | cold-cache min=512923 µs |
| M=1048576 B | threads=16 | cold-cache min=500631 µs |

### Badanie latencji

Bez czyszczenia cachea latencja z zasady rośnie wraz ze wzrostem wątków.
| Rozmiar (M) | Wątki (threads) | Czas minimalny (min) |
| --- | --- | --- |
| M=512 B | threads=1 | cache min avg-read=0.699 µs |
| M=512 B | threads=2 | cache min avg-read=0.746 µs |
| M=512 B | threads=4 | cache min avg-read=0.799 µs |
| M=512 B | threads=8 | cache min avg-read=0.755 µs |
| M=512 B | threads=16 | cache min avg-read=1.096 µs |
| M=4096 B | threads=1 | cache min avg-read=1.050 µs |
| M=4096 B | threads=2 | cache min avg-read=1.196 µs |
| M=4096 B | threads=4 | cache min avg-read=1.272 µs |
| M=4096 B | threads=8 | cache min avg-read=1.434 µs |
| M=4096 B | threads=16 | cache min avg-read=2.264 µs |
| M=16384 B | threads=1 | cache min avg-read=2.825 µs |
| M=16384 B | threads=2 | cache min avg-read=2.885 µs |
| M=16384 B | threads=4 | cache min avg-read=3.078 µs |
| M=16384 B | threads=8 | cache min avg-read=3.809 µs |
| M=16384 B | threads=16 | cache min avg-read=8.236 µs |
| M=65536 B | threads=1 | cache min avg-read=8.565 µs |
| M=65536 B | threads=2 | cache min avg-read=9.229 µs |
| M=65536 B | threads=4 | cache min avg-read=9.920 µs |
| M=65536 B | threads=8 | cache min avg-read=14.981 µs |
| M=65536 B | threads=16 | cache min avg-read=30.619 µs |
| M=1048576 B | threads=1 | cache min avg-read=158.855 µs |
| M=1048576 B | threads=2 | cache min avg-read=164.929 µs |
| M=1048576 B | threads=4 | cache min avg-read=169.296 µs |
| M=1048576 B | threads=8 | cache min avg-read=291.001 µs |
| M=1048576 B | threads=16 | cache min avg-read=621.211 µs |

Z czyszczeniem cachea latencja też najczęściej rośnie wraz ze zwiększaniem liczby wątków.
| Rozmiar (M) | Wątki (threads) | Czas minimalny (min) |
| --- | --- | --- |
| M=512 B | threads=1 | cold-cache min avg-read=12.304 µs |
| M=512 B | threads=2 | cold-cache min avg-read=11.767 µs |
| M=512 B | threads=4 | cold-cache min avg-read=12.735 µs |
| M=512 B | threads=8 | cold-cache min avg-read=15.248 µs |
| M=512 B | threads=16 | cold-cache min avg-read=21.303 µs |
| M=4096 B | threads=1 | cold-cache min avg-read=92.683 µs |
| M=4096 B | threads=2 | cold-cache min avg-read=112.451 µs |
| M=4096 B | threads=4 | cold-cache min avg-read=136.978 µs |
| M=4096 B | threads=8 | cold-cache min avg-read=121.349 µs |
| M=4096 B | threads=16 | cold-cache min avg-read=165.227 µs |
| M=16384 B | threads=1 | cold-cache min avg-read=126.782 µs |
| M=16384 B | threads=2 | cold-cache min avg-read=134.926 µs |
| M=16384 B | threads=4 | cold-cache min avg-read=145.198 µs |
| M=16384 B | threads=8 | cold-cache min avg-read=157.190 µs |
| M=16384 B | threads=16 | cold-cache min avg-read=175.806 µs |
| M=65536 B | threads=1 | cold-cache min avg-read=189.599 µs |
| M=65536 B | threads=2 | cold-cache min avg-read=207.314 µs |
| M=65536 B | threads=4 | cold-cache min avg-read=202.391 µs |
| M=65536 B | threads=8 | cold-cache min avg-read=206.481 µs |
| M=65536 B | threads=16 | cold-cache min avg-read=218.147 µs |
| M=1048576 B | threads=1 | cold-cache min avg-read=1050.562 µs |
| M=1048576 B | threads=2 | cold-cache min avg-read=1077.844 µs |
| M=1048576 B | threads=4 | cold-cache min avg-read=1142.801 µs |
| M=1048576 B | threads=8 | cold-cache min avg-read=1237.139 µs |
| M=1048576 B | threads=16 | cold-cache min avg-read=1768.545 µs |


## Przepustowość
testy na 2 GB

Bez czyszczenia cachea:
| Rozmiar (M) | Przed zmianą | Po zmianie |
| --- | --- | --- |
| M=512 B | read min=4924002 µs | mmap min=3534361 µs |
| M=4096 B | read min=3520855 µs | mmap min=3193777 µs |
| M=16384 B | read min=3361478 µs | mmap min=3156374 µs |
| M=65536 B | read min=3336747 µs | mmap min=3146178 µs |
| M=1048576 B | read min=3297555 µs | mmap min=3133125 µs |
| M=4194304 B | read min=3295846 µs | mmap min=3125806 µs |
| M=16777216 B | read min=3320071 µs | mmap min=3139622 µs |

Przy małych blokach mmap najbardziej pomaga.

Z czyszczeniem cachea:
| Rozmiar (M) | Przed zmianą | Po zmianie |
| --- | --- | --- |
| M=512 B | pread min=5786498 µs | mmap min=4317164 µs |
| M=4096 B | pread min=4234810 µs | mmap min=4196396 µs |
| M=16384 B | pread min=4553859 µs | mmap min=3950542 µs |
| M=65536 B | pread min=4272524 µs | mmap min=3951184 µs |
| M=1048576 B | pread min=4706441 µs | mmap min=4552227 µs |
| M=4194304 B | pread min=4559662 µs | mmap min=3821994 µs |
| M=16777216 B | pread min=4310917 µs | mmap min=4013798 µs |

Czy jest jakiś schemat odczytywania pliku, gdzie mmap zmniejszy obciążenie procesora ponad read?

Tak np. odczytywanie tego samego wiele razy krótkiego fragmentu dużego pliku, bo po możliwym jednym pagefaulcie potem czytam bezpośrednio ten fragment bez ponownych syscalli które przełączają context.

Jak wpłynie na latencję posiadanie pliku w pamięci systemu operacyjnego (file cache)?

Dla plików w całości mieszczących się w RAM powinna być mniejsza, bo mimo że i tak może musieć odbyć się mapowanie to jest to różnica między sięganiem do dysku, a tego nie robieniem. (nie wiem czy to jest podchwytliwe pytanie?)



## Pipeline
test na 2gb
| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=65536 B | K=1 | min=3680170 µs |
| M=65536 B | K=10 | min=3888264 µs |
| M=65536 B | K=100 | min=4089995 µs |
| M=65536 B | K=1000 | min=6996593 µs |
| M=262144 B | K=1 | min=3329593 µs |
| M=262144 B | K=10 | min=3311680 µs |
| M=262144 B | K=100 | min=3369675 µs |
| M=262144 B | K=1000 | min=4116075 µs |
| M=1048576 B | K=1 | min=3160719 µs |
| M=1048576 B | K=10 | min=3187546 µs |
| M=1048576 B | K=100 | min=3208377 µs |
| M=1048576 B | K=1000 | min=3404411 µs |
| M=4194304 B | K=1 | min=3130690 µs |
| M=4194304 B | K=10 | min=3144642 µs |
| M=4194304 B | K=100 | min=3135875 µs |
| M=4194304 B | K=1000 | min=3181223 µs |


Szukanie K dla mieszczącego się w ramie

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=1 | min=3139726 µs |
| M=1048576 B | K=5 | min=3164231 µs |
| M=1048576 B | K=10 | min=3182211 µs |
| M=1048576 B | K=20 | min=3215488 µs |
| M=1048576 B | K=50 | min=3261756 µs |
| M=1048576 B | K=100 | min=3304673 µs |
| M=1048576 B | K=200 | min=3342143 µs |
| M=1048576 B | K=500 | min=3360847 µs |
| M=1048576 B | K=1000 | min=3431872 µs |

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=1 | min=3119754 µs |
| M=1048576 B | K=2 | min=3145404 µs |
| M=1048576 B | K=3 | min=3179887 µs |
| M=1048576 B | K=4 | min=3167832 µs |
| M=1048576 B | K=5 | min=3214350 µs |

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=1000 | min=3393997 µs |
| M=1048576 B | K=2000 | min=3660344 µs |
| M=1048576 B | K=3000 | min=3895904 µs |
| M=1048576 B | K=4000 | min=4115735 µs |
| M=1048576 B | K=5000 | min=4331638 µs |

Wygląda na to że mam szybki ram albo wolny komputer i K_opt = 1 bo nie mogę ukryc kosztu obliczeń w IO

Dla 512 Ramu mamy już 

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=1 | min=3307654 µs |
| M=1048576 B | K=5 | min=3300723 µs |
| M=1048576 B | K=50 | min=3351042 µs |
| M=1048576 B | K=100 | min=3397300 µs |
| M=1048576 B | K=200 | min=3467855 µs |
| M=1048576 B | K=500 | min=4368019 µs |
| M=1048576 B | K=1000 | min=3934383 µs |

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=1 | min=3253520 µs |
| M=1048576 B | K=10 | min=3265354 µs |
| M=1048576 B | K=20 | min=3247835 µs |
| M=1048576 B | K=30 | min=3262596 µs |
| M=1048576 B | K=40 | min=3284078 µs |
| M=1048576 B | K=50 | min=3281418 µs |
| M=1048576 B | K=60 | min=3262691 µs |
| M=1048576 B | K=70 | min=3276033 µs |
| M=1048576 B | K=80 | min=3291282 µs |
| M=1048576 B | K=90 | min=3279331 µs |
| M=1048576 B | K=100 | min=3292046 µs |

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=150 | min=3321546 µs |
| M=1048576 B | K=200 | min=3533001 µs |
| M=1048576 B | K=300 | min=3528237 µs |
| M=1048576 B | K=400 | min=3419974 µs |
| M=1048576 B | K=500 | min=3449662 µs |
| M=1048576 B | K=700 | min=3439776 µs |
| M=1048576 B | K=1000 | min=3450137 µs |
| M=1048576 B | K=1500 | min=3573480 µs |

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=1 | min=3218698 µs |
| M=1048576 B | K=100 | min=3269209 µs |
| M=1048576 B | K=150 | min=3274462 µs |
| M=1048576 B | K=200 | min=3291274 µs |
| M=1048576 B | K=300 | min=3320835 µs |
| M=1048576 B | K=400 | min=3321847 µs |
| M=1048576 B | K=700 | min=3371470 µs |
| M=1048576 B | K=1000 | min=3427372 µs |
| M=1048576 B | K=1500 | min=3560330 µs |

| Rozmiar (M) | K | czas obliczeń |
| --- | --- | --- |
| M=1048576 B | K=21 | min=3253246 µs |
| M=1048576 B | K=23 | min=3268640 µs |
| M=1048576 B | K=25 | min=3282429 µs |
| M=1048576 B | K=27 | min=3275655 µs |
| M=1048576 B | K=29 | min=3213893 µs |

Przez zmienność obliczeń trudno ustalić K optymalne, ale przyjmijmy je koło 20 kilku


Dla jednego obliczacza i wielu czytelników (M=1048576 B, K=1, 1 obliczacz)
| Czytelnicy | Obliczacze | czas obliczeń |
| --- | --- | --- |
| readers=1 | computers=1 | min=3217814 µs |
| readers=2 | computers=1 | min=3263716 µs |
| readers=4 | computers=1 | min=3228835 µs |
| readers=8 | computers=1 | min=3233692 µs |

Dla jednego czytelnika i wielu obliczaczy M=1048576 B, K=1000
| Czytelnicy | Obliczacze | czas obliczeń |
| --- | --- | --- |
| readers=1 | computers=1 | min=3460854 µs |
| readers=1 | computers=2 | min=1844325 µs |
| readers=1 | computers=4 | min=1072726 µs |
| readers=1 | computers=8 | min=886317 µs |

Wynika z tego że dla 2gb pliku i bloku 1048576 liczą się tylko obliczenia które można skracać dodając więcej wątków liczących.