import http from "k6/http"
import { sleep } from "k6"

export default function () {
  const q1 = `   
    query Request {
      query {
        locations {
            name
            id
        }
      }
    }`
  http.post("http://localhost:3000/graphql", JSON.stringify({ query: q1 }))
  // sleep(0.1)
  const q2 = `   
    query Request {
      query {
        locations {
          menus {
            date
            meals {
              mealType
              sections {
                name
                foodItems {
                    name
                }
              }
            }
          }
        }
      }
    }`
  // sleep(1)
  http.post("http://localhost:3000/graphql", JSON.stringify({ query: q2 }))
  http.put("http://localhost:3000/request_refresh")
}
