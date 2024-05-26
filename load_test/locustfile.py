from locust import HttpUser, task
from random import randint

class LoadTest(HttpUser):
    cities = ["Trondheim", "Oslo", "Bergen", "Stavanger", "Tromsø", "Kristiansand", "Bodø", "Drammen", "Fredrikstad", "Skien", "Sandnes", "Sarpsborg", "Ålesund", "Sandefjord", "Haugesund", "Tønsberg", "Moss", "Porsgrunn", "Arendal", "Molde", "Kongsberg", "Horten", "Harstad", "Larvik", "Askøy", "Ytrebygda", "Halden", "Steinkjer", "Lillehammer", "Mandal", "Gjøvik", "Narvik", "Kristiansund", "Ås", "Hamar", "Hønefoss", "Elverum", "Mysen", "Førde", "Kongsvinger", "Leirvik", "Vennesla", "Lillestrøm", "Grimstad", "Mo i Rana", "Nesoddtangen", "Lørenskog", "Verdal", "Kopervik", "Åkrehamn", "Ski", "Drøbak", "Korsvik", "Lillesand", "Namsos", "Jørpeland", "Fauske", "Ulsteinvik", "Råholt", "Florø", "Svelvik", "Brumunddal", "Bryne", "Stjørdalshalsen", "Notodden", "Sola", "Brekstad", "Volda", "Sogndal", "Leknes", "Skiptvet", "Åkrehamn", "Ski", "Drøbak", "Korsvik", "Lillesand", "Namsos", "Jørpeland", "Fauske", "Ulsteinvik", "Råholt", "Florø", "Svelvik", "Brumunddal", "Bryne", "Stjørdalshalsen", "Notodden", "Sola", "Brekstad", "Volda", "Sogndal", "Leknes", "Skiptvet", "Åkreham"]

    @task
    def alerts(self):
        city = self.cities[randint(0, len(self.cities) - 1)]
        self.client.get(f"api/alerts?location={city}")

    @task
    def nowcasts(self):
        city = self.cities[randint(0, len(self.cities) - 1)]
        self.client.get(f"api/nowcasts?location={city}")
