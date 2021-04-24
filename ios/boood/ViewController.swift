//
//  ViewController.swift
//  boood
//
//  Created by Alagris on 23/04/2021.
//  Copyright © 2021 alagris. All rights reserved.
//

import UIKit

class ViewController: UIViewController {
	
	override func viewDidLoad() {
		super.viewDidLoad()
		// Do any additional setup after loading the view.
	}
	
	@IBAction func queryServer(_ sender: UIButton) {
		
//		let params = ["username":"john", "password":"123456"] as Dictionary<String, String>
//
		var request = URLRequest(url: URL(string: "https://en.wikipedia.org/w/api.php?action=abusefiltercheckmatch&filter=!(%22autoconfirmed%22%20in%20user_groups)&rcid=15&format=json")!)
		request.httpMethod = "GET"
//		request.httpBody = try? JSONSerialization.data(withJSONObject: params, options: [])
		request.addValue("application/json", forHTTPHeaderField: "Content-Type")
		
		let session = URLSession.shared
		let task = session.dataTask(with: request, completionHandler: { data, response, error -> Void in
			print(response!)
			do {
				let json = try JSONSerialization.jsonObject(with: data!) as! Dictionary<String, AnyObject>
				print(json)
			} catch {
				print("error")
			}
		})
		
		task.resume()
		
		
	}
	
}

