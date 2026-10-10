# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `197e855` on `m3` (2026-10-10T06:52:15Z) · profile release · laya feature false
- box state (Issue 021): start power=AC Power mode=high load=4.00 swap=2405M · end power=AC Power mode=high load=2.50 swap=2381M — latency QUOTABLE
- laya device posture: n/a (compiled without laya-riir)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: off (pass --bekko to add the comparison lane; needs the bekko venv — Bench 103)
- agentjev lane: off (pass --agentjev to add the comparison lane; needs their jev_service on AGENTJEV_SERVE_URL — .issues/025)
- clef lane: off (pass --clef to add the comparison lane; the hosted posture needs the owner's Workers AI creds behind the loopback forwarder, or serve the open weights locally — plan 011 Phase A + .research/007)
- drex lane: off (pass --drex to add the comparison lane; needs their service on DREX_SERVE_URL, default http://127.0.0.1:8000 — Issue 073)
- d1 lane: off (pass --d1 to add the comparison lane; needs the reference server on D1_SERVE_URL, default http://127.0.0.1:8078 — Issue 078)
- pplx lane: off (pass --pplx to add the comparison lane; serve their autojev server on PPLX_SERVE_URL, default http://127.0.0.1:8793 — Issue 082)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): ON — cal-selected per suite (scale 0/1/4/16/32/64 × α observed-laplace/fixed-1, promotion bar +5 pt over off on the stratified slice; + noul polarity per domain on noul suites; + bag/pair view on multi-field states; count tables from TRAIN rows only, uncapped). Transductive column: TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc
- option-conditioned tables (issue 038 T7b): ON — cal-selected per suite (scale 0/0.25/0.5/1/2/4/8, promotion bar +5 pt over off on the stratified slice; one contrastive table per (question id, gold option) from the TRAIN rows, events filtered to the corpus pool; every question kind armed — typed_decisions is the target suite)
- NBSVM ridge readout (issue 038 T7a): ON — cal-selected per suite (scale 0/0.5/1/2/4/8 × λ 10 fixed, promotion bar +5 pt over off on the stratified slice; NBSVM closed-form ridge, k=2048, per-class NB log-count ratios, fit-time self-calibrated margin temperature)
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved; --drafter-fix per_byte: the drafter-only correction mode armed (issue 079 tuning axis; EngineConfig::drafter_fix, issue 036 T2)
- sampling: test sample = label-STRATIFIED round-robin over the whole test split (budget = the registry test cap; first-appearance label order, dataset order within each label, deterministic, no RNG); cal slice = the same law over the train rows; corpus pool = the train rows MINUS the cal front (excluded by construction, not position); budget-0 suites unchanged (identity split) — Issue 039 T2
- readout: shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side arming lever overfit the narrow suites' cal slice — emotion's test G1 regressed — and coincided with Dispatch on the wide suites it targeted; the wide-label G1 gap was closed by T2's stratified cal slice). The candidate table is recorded per suite (report-only); EngineConfig::readout stays the opt-in knob
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: synthetic fixtures (semantic_defects, code_fixtures): in-process fixtures with programmatic gold; the six Issue-004 harness families were retired 2026-10-02 (owner call — at-chance on the modelless lane at the honest populations)

## s1mb_choice — 4329 cases / 4329 questions

**slice-integrity: test 4330 (fnv1a64-3d5a30a2826dd8dc) · cal 200 (fnv1a64-4366c2ff02074485) · pool 4130 (fnv1a64-7c46a2ccb379573c) · cal missing 642: virtual_card_not_working,disposable_card_limits,pending_cash_withdrawal,pending_card_payment,verify_source_of_funds,declined_card_payment,card_payment_wrong_exchange_rate,wrong_exchange_rate_for_cash_withdrawal,balance_not_updated_after_bank_transfer,failed_transfer,cancel_transfer,top_up_by_bank_transfer_charge,beneficiary_not_allowed,card_payment_not_recognised,transfer_into_account,cash_withdrawal_charge,Refund_not_showing_up,pending_top_up,card_about_to_expire,exchange_charge,transfer_fee_charged,fiat_currency_support,card_delivery_estimate,translate,cancel,todo_list,last_maintenance,pto_balance,directions,confirm_reservation,order_status,play_music,oos,calculator,travel_suggestion,shopping_list,change_language,timezone,yes,book_flight,payday,credit_limit_change,calendar_update,improve_credit_score,pin_change,update_playlist,cancel_reservation,time,flip_coin,find_phone,redeem_rewards,change_ai_name,min_payment,spelling,schedule_meeting,international_fees,reminder,book_hotel,current_location,goodbye,insurance_change,new_card,fun_fact,transfer,freeze_account,schedule_maintenance,gas,vaccines,distance,travel_notification,who_made_you,next_holiday,bill_balance,how_old_are_you,rollover_401k,direct_deposit,sync_device,uber,order_checks,alarm,income,reset_settings,meaning_of_life,make_call,insurance,restaurant_suggestion,interest_rate,bill_due,who_do_you_work_for,roll_dice,damaged_card,what_are_your_hobbies,lost_luggage,text,international_visa,application_status,what_can_i_ask_you,replacement_card_duration,bank_routing_number,company_name,swift_bic,cvv,account_number,postcode,ipv6,unique_identifier,pin,coordinate,url,date_time,-47,-4,14,34,74,24,28,5760,5756,5750,5765,15,33,-2,8,12,19,13,29,9,16,52,22,-5,17,72,80,75,78,103,98,48,200,150,180,196,64,114,59,84,38,63,43,32,21,20,-18,27,7,149,197,172,147,100,88,93,-1,10,6,-3,350,397,400,375,248,254,255,251,6580,6598,6600,6593,293,305,288,300,13997,14000,13993,14007,-33,475,504,501,500,26,11,40,90,70,89,1198,1194,1202,1173,297,310,-15,-10,71,121,-21,36,31,122,128,125,45,190,207,47,39,92,130,76,195,41,50,46,280,283,273,44,18,-8,62,30,5,55,play game,iot wemo on,iot coffee,iot hue lightchange,audio volume mute,transport traffic,calendar query,qa factoid,cooking recipe,play audiobook,iot cleaning,social post,weather query,transport taxi,alarm remove,iot hue lightdim,general quirky,music likeness,general negate,alarm query,recommendation events,qa currency,social query,audio volume up,general affirm,datetime query,play music,alarm set,email sendemail,lists createoradd,datetime convert,play podcasts,calendar set,transport query,general dontcare,email querycontact,qa stock,general commandstop,iot wemo off,general confirm,general repeat,music settings,general explain,lists query,recommendation locations,general joke,iot hue lightup,iot hue lightoff,iot hue lighton,news query,audio volume down,email query,qa maths,takeaway query,qa definition,general praise,takeaway order,play radio,lists remove,calendar remove,music query,recommendation movies,email addcontact,transport ticket,world,business,sci_tech,sports,monitor,contain,close_benign,investigate,hold,reject,approve,manual_review,request_information,answer_directly,escalate_to_human,close_no_action,execute_refund,technical,account,delivery,refund,billing,observe,stop,continue,human_review,partial,success,harmful,failure,iot_wemo_off,qa_stock,qa_maths,recommendation_movies,audio_volume_up,play_audiobook,recommendation_locations,cooking_recipe,takeaway_order,play_radio,iot_hue_lightchange,datetime_convert,general_joke,cooking_query,qa_currency,lists_remove,takeaway_query,weather_query,transport_traffic,music_dislikeness,music_query,iot_hue_lightdim,email_sendemail,iot_hue_lightup,transport_query,social_query,qa_factoid,email_querycontact,qa_definition,calendar_query,iot_hue_lighton,audio_volume_down,lists_query,calendar_set,social_post,news_query,audio_volume_other,alarm_remove,transport_ticket,iot_coffee,general_greet,iot_hue_lightoff,iot_wemo_on,play_game,alarm_set,email_query,datetime_query,alarm_query,calendar_remove,music_settings,iot_cleaning,music_likeness,transport_taxi,general_quirky,play_podcasts,lists_createoradd,email_addcontact,audio_volume_mute,recommendation_events,recommendation,transport,news,play,lists,iot,datetime,music,qa,general,social,takeaway,cooking,audio,a,b,messaging,people,calling,recipes,IN:GET_SUNSET,IN:LIKE_MUSIC,IN:GET_REMINDER_LOCATION,IN:GET_RECIPES,IN:PLAY_MUSIC,IN:REPEAT_ALL_MUSIC,IN:UPDATE_ALARM,IN:GET_TIMER,IN:CREATE_PLAYLIST_MUSIC,IN:DELETE_REMINDER,IN:PAUSE_MUSIC,IN:SET_DEFAULT_PROVIDER_CALLING,IN:SWITCH_CALL,IN:SET_UNAVAILABLE,IN:GET_EDUCATION_DEGREE,IN:SUBTRACT_TIME_TIMER,IN:UPDATE_REMINDER_TODO,IN:GET_MAJOR,IN:GET_TRACK_INFO_MUSIC,IN:GET_ATTENDEE_EVENT,IN:UPDATE_METHOD_CALL,IN:DELETE_PLAYLIST_MUSIC,IN:GET_AVAILABILITY,IN:RESUME_TIMER,IN:GET_SUNRISE,IN:GET_EMPLOYMENT_TIME,IN:GET_REMINDER_DATE_TIME,IN:CREATE_REMINDER,IN:SKIP_TRACK_MUSIC,IN:GET_REMINDER,IN:SET_RSVP_YES,IN:GET_AGE,IN:GET_LIFE_EVENT,IN:FAST_FORWARD_MUSIC,IN:STOP_SHUFFLE_MUSIC,IN:GET_DATE_TIME_EVENT,IN:GET_CALL,IN:REWIND_MUSIC,IN:MERGE_CALL,IN:DELETE_ALARM,IN:REPEAT_ALL_OFF_MUSIC,IN:GET_CALL_CONTACT,IN:UNLOOP_MUSIC,IN:SET_DEFAULT_PROVIDER_MUSIC,IN:RESUME_CALL,IN:CREATE_CALL,IN:START_SHUFFLE_MUSIC,IN:GET_CALL_TIME,IN:GET_CONTACT_METHOD,IN:GET_UNDERGRAD,IN:END_CALL,IN:GET_LANGUAGE,IN:GET_GENDER,IN:SHARE_EVENT,IN:RESTART_TIMER,IN:SET_RSVP_INTERESTED,IN:GET_MESSAGE,IN:QUESTION_NEWS,IN:IGNORE_CALL,IN:GET_DETAILS_NEWS,IN:DISLIKE_MUSIC,IN:GET_INFO_RECIPES,IN:QUESTION_MUSIC,IN:ADD_TIME_TIMER,IN:GET_JOB,IN:GET_MESSAGE_CONTACT,IN:PREVIOUS_TRACK_MUSIC,IN:ANSWER_CALL,IN:SEND_MESSAGE,IN:GET_EMPLOYER,IN:SET_AVAILABLE,IN:UPDATE_REMINDER_LOCATION,IN:GET_MUTUAL_FRIENDS,IN:SILENCE_ALARM,IN:REMOVE_FROM_PLAYLIST_MUSIC,IN:PLAY_MEDIA,IN:HOLD_CALL,IN:UPDATE_REMINDER,IN:GET_STORIES_NEWS,IN:GET_EVENT,IN:RESUME_MUSIC,IN:GET_CATEGORY_EVENT,IN:HELP_REMINDER,IN:REPLAY_MUSIC,IN:CREATE_ALARM,IN:FOLLOW_MUSIC,IN:GET_AIRQUALITY,IN:GET_LYRICS_MUSIC,IN:CANCEL_CALL,IN:GET_REMINDER_AMOUNT,IN:PREFER,IN:UPDATE_TIMER,IN:GET_WEATHER,IN:GET_INFO_CONTACT,IN:UPDATE_REMINDER_DATE_TIME,IN:SNOOZE_ALARM,IN:CANCEL_MESSAGE,IN:DELETE_TIMER,IN:GET_LIFE_EVENT_TIME,IN:SET_RSVP_NO,IN:GET_LOCATION,IN:GET_CONTACT,IN:IS_TRUE_RECIPES,IN:ADD_TO_PLAYLIST_MUSIC,IN:GET_GROUP,IN:PAUSE_TIMER,IN:GET_ALARM,IN:DISPREFER,IN:UPDATE_CALL,IN:LOOP_MUSIC,IN:CREATE_TIMER,IN:GET_EDUCATION_TIME,IN:STOP_MUSIC,option_2,option_3,option_4,option_5,option_6,option_7,option_8,option_9,option_10,option_11,option_12,option_13,option_14,option_15,positive,mixed,negative,no_impact,method,background,result,BookAppointment,NONE,FindProvider,SearchHotel,ReserveHotel,FindMovies,PlayMovie,FindTrains,GetTrainTickets,MakePayment,RequestPayment,GetTimesForMovie,BuyMovieTickets,ReserveRestaurant,FindRestaurants,BookHouse,SearchHouse,SearchOnewayFlight,SearchRoundtripFlights,LookupMusic,PlayMedia,GetAlarms,AddAlarm,BuyBusTicket,FindBus,ReserveCar,GetCarsAvailable,BuyEventTickets,FindEvents,GetWeather,GetRide,FindAttractions,ScheduleVisit,FindHomeByArea,ShareLocation,left,upper-right,right,above,upper-left,lower-right,lower-left,below,overlap,SYM,VERB,ADJ,ADV,INTJ,PART,PRON,X,AUX,PROPN,DET,NOUN,PUNCT,CCONJ,ADP,NUM,SCONJ,mechanic,hairdresser,driver,teacher,nurse,assistant,construction worker,tailor,physician,analyst,supervisor,attendant,guard,housekeeper,accountant,janitor,clerk,counselor,laborer,secretary,writer,farmer,sheriff,lawyer,baker,carpenter,developer,cleaner,cook,salesperson,editor,receptionist,auditor,manager,designer · pool missing 464: getting_spare_card,verify_my_identity,virtual_card_not_working,disposable_card_limits,reverted_card_payment?,pending_cash_withdrawal,get_physical_card,lost_or_stolen_card,top_up_by_card_charge,extra_charge_on_statement,automatic_top_up,get_disposable_virtual_card,transfer_not_received_by_recipient,request_refund,pending_card_payment,declined_transfer,age_limit,card_payment_fee_charged,verify_source_of_funds,declined_card_payment,card_payment_wrong_exchange_rate,wrong_amount_of_cash_received,atm_support,transaction_charged_twice,wrong_exchange_rate_for_cash_withdrawal,supported_cards_and_currencies,balance_not_updated_after_bank_transfer,card_swallowed,failed_transfer,cancel_transfer,direct_debit_payment_not_recognised,top_up_limits,exchange_via_app,passcode_forgotten,top_up_by_cash_or_cheque,top_up_by_bank_transfer_charge,beneficiary_not_allowed,compromised_card,unable_to_verify_identity,card_not_working,exchange_rate,card_payment_not_recognised,transfer_into_account,cash_withdrawal_charge,card_arrival,card_acceptance,activate_my_card,transfer_timing,terminate_account,topping_up_by_card,cash_withdrawal_not_recognised,getting_virtual_card,country_support,Refund_not_showing_up,balance_not_updated_after_cheque_or_cash_deposit,change_pin,pending_transfer,edit_personal_details,verify_top_up,pending_top_up,why_verify_identity,card_about_to_expire,exchange_charge,transfer_fee_charged,top_up_failed,fiat_currency_support,card_delivery_estimate,rewards_balance,translate,cancel,maybe,restaurant_reviews,repeat,todo_list,flight_status,last_maintenance,pto_balance,directions,restaurant_reservation,confirm_reservation,order_status,tell_joke,oos,calculator,gas_type,travel_suggestion,share_location,shopping_list,change_language,timezone,yes,book_flight,payday,no,meeting_schedule,do_you_have_pets,order,credit_limit_change,recipe,calendar_update,improve_credit_score,pin_change,update_playlist,cancel_reservation,date,measurement_conversion,definition,oil_change_how,whisper_mode,flip_coin,find_phone,todo_list_update,redeem_rewards,change_user_name,change_ai_name,min_payment,spelling,how_busy,change_accent,schedule_meeting,food_last,international_fees,book_hotel,where_are_you_from,change_volume,are_you_a_bot,tire_change,ingredients_list,current_location,goodbye,insurance_change,w2,pto_used,new_card,traffic,balance,fun_fact,ingredient_substitution,transfer,freeze_account,schedule_maintenance,thank_you,what_is_your_name,gas,vaccines,accept_reservations,transactions,cook_time,distance,routing,report_lost_card,next_song,carry_on,what_song,travel_notification,apr,nutrition_info,who_made_you,next_holiday,bill_balance,reminder_update,how_old_are_you,mpg,rollover_401k,account_blocked,plug_type,card_declined,smart_home,meal_suggestion,direct_deposit,sync_device,pay_bill,uber,pto_request_status,taxes,order_checks,greeting,income,spending_history,calories,reset_settings,expiration_date,tire_pressure,meaning_of_life,make_call,insurance,restaurant_suggestion,interest_rate,credit_limit,bill_due,who_do_you_work_for,report_fraud,roll_dice,change_speed,damaged_card,shopping_list_update,what_are_your_hobbies,lost_luggage,oil_change_when,text,travel_alert,car_rental,international_visa,application_status,jump_start,credit_score,what_can_i_ask_you,replacement_card_duration,pto_request,national_id,company_name,account_number,password,url,-47,-4,14,34,24,5760,5756,5750,5765,33,-2,8,12,29,16,52,-5,17,72,80,78,103,98,48,200,150,180,196,64,114,59,84,38,63,43,32,21,20,-18,27,149,197,172,147,88,93,-1,-3,350,397,400,375,248,254,255,251,6580,6598,6600,6593,293,305,288,13997,14000,13993,14007,-33,475,504,501,500,11,40,90,70,89,1198,1194,1202,1173,297,310,-15,-10,71,121,-21,36,122,128,125,190,207,47,39,92,130,76,195,41,46,283,273,44,18,-8,62,5,55,play game,iot coffee,transport traffic,calendar query,qa factoid,play audiobook,iot cleaning,alarm remove,general negate,recommendation events,alarm set,play podcasts,transport query,general commandstop,iot wemo off,music settings,general explain,lists query,general joke,iot hue lightup,email query,takeaway query,play radio,lists remove,music query,recommendation movies,close_no_action,delivery,stop,harmful,qa_stock,recommendation_movies,audio_volume_up,play_audiobook,recommendation_locations,cooking_recipe,datetime_convert,cooking_query,qa_currency,weather_query,music_query,iot_hue_lightdim,iot_hue_lightup,qa_definition,audio_volume_down,general_greet,iot_hue_lightoff,alarm_set,general_quirky,play_podcasts,email_addcontact,cooking,IN:LIKE_MUSIC,IN:GET_RECIPES,IN:GET_TIMER,IN:CREATE_PLAYLIST_MUSIC,IN:PAUSE_MUSIC,IN:SET_DEFAULT_PROVIDER_CALLING,IN:SWITCH_CALL,IN:SET_UNAVAILABLE,IN:GET_EDUCATION_DEGREE,IN:GET_MAJOR,IN:GET_TRACK_INFO_MUSIC,IN:GET_ATTENDEE_EVENT,IN:UPDATE_METHOD_CALL,IN:GET_AVAILABILITY,IN:GET_EMPLOYMENT_TIME,IN:GET_REMINDER_DATE_TIME,IN:CREATE_REMINDER,IN:SKIP_TRACK_MUSIC,IN:GET_AGE,IN:GET_LIFE_EVENT,IN:STOP_SHUFFLE_MUSIC,IN:GET_CALL,IN:MERGE_CALL,IN:DELETE_ALARM,IN:UNLOOP_MUSIC,IN:SET_DEFAULT_PROVIDER_MUSIC,IN:RESUME_CALL,IN:CREATE_CALL,IN:GET_CALL_TIME,IN:GET_CONTACT_METHOD,IN:GET_UNDERGRAD,IN:GET_GENDER,IN:SHARE_EVENT,IN:RESTART_TIMER,IN:SET_RSVP_INTERESTED,IN:QUESTION_NEWS,IN:GET_DETAILS_NEWS,IN:ADD_TIME_TIMER,IN:GET_MESSAGE_CONTACT,IN:ANSWER_CALL,IN:SET_AVAILABLE,IN:UPDATE_REMINDER_LOCATION,IN:REMOVE_FROM_PLAYLIST_MUSIC,IN:PLAY_MEDIA,IN:HOLD_CALL,IN:GET_STORIES_NEWS,IN:GET_EVENT,IN:FOLLOW_MUSIC,IN:GET_AIRQUALITY,IN:CANCEL_CALL,IN:GET_REMINDER_AMOUNT,IN:GET_WEATHER,IN:UPDATE_REMINDER_DATE_TIME,IN:CANCEL_MESSAGE,IN:IS_TRUE_RECIPES,IN:GET_GROUP,IN:PAUSE_TIMER,IN:DISPREFER,IN:CREATE_TIMER,option_9,option_10,option_11,option_12,option_13,option_14,option_15,mixed,SearchHotel,MakePayment,RequestPayment,GetTimesForMovie,BuyMovieTickets,FindRestaurants,SearchHouse,SearchOnewayFlight,LookupMusic,AddAlarm,GetWeather,GetRide,ShareLocation,teacher,supervisor,clerk,lawyer,baker,carpenter,developer,auditor · OK**

**corpus cap:** 16 (registry)

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.1550 |
| 1 | observed-laplace | — | bag | 0.1850 |
| 4 | observed-laplace | — | bag | 0.1800 |
| 16 | observed-laplace | — | bag | 0.1750 |
| 32 | observed-laplace | — | bag | 0.1750 |
| 64 | observed-laplace | — | bag | 0.1800 |
| 1 | fixed-1 | — | bag | 0.1700 |
| 4 | fixed-1 | — | bag | 0.1750 |
| 16 | fixed-1 | — | bag | 0.1650 |
| 32 | fixed-1 | — | bag | 0.1600 |
| 64 | fixed-1 | — | bag | 0.1600 |

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1354 · max_prob 0.0950 · inv_entropy 0.1523

⛔ **corpus fallback (Issue 039 guard):** 464 option label(s) with NO train docs in the corpus pool — self-doc fallback only: getting_spare_card, verify_my_identity, virtual_card_not_working, disposable_card_limits, reverted_card_payment?, pending_cash_withdrawal, get_physical_card, lost_or_stolen_card, top_up_by_card_charge, extra_charge_on_statement, automatic_top_up, get_disposable_virtual_card, transfer_not_received_by_recipient, request_refund, pending_card_payment, declined_transfer, age_limit, card_payment_fee_charged, verify_source_of_funds, declined_card_payment, card_payment_wrong_exchange_rate, wrong_amount_of_cash_received, atm_support, transaction_charged_twice, wrong_exchange_rate_for_cash_withdrawal, supported_cards_and_currencies, balance_not_updated_after_bank_transfer, card_swallowed, failed_transfer, cancel_transfer, direct_debit_payment_not_recognised, top_up_limits, exchange_via_app, passcode_forgotten, top_up_by_cash_or_cheque, top_up_by_bank_transfer_charge, beneficiary_not_allowed, compromised_card, unable_to_verify_identity, card_not_working, exchange_rate, card_payment_not_recognised, transfer_into_account, cash_withdrawal_charge, card_arrival, card_acceptance, activate_my_card, transfer_timing, terminate_account, topping_up_by_card, cash_withdrawal_not_recognised, getting_virtual_card, country_support, Refund_not_showing_up, balance_not_updated_after_cheque_or_cash_deposit, change_pin, pending_transfer, edit_personal_details, verify_top_up, pending_top_up, why_verify_identity, card_about_to_expire, exchange_charge, transfer_fee_charged, top_up_failed, fiat_currency_support, card_delivery_estimate, rewards_balance, translate, cancel, maybe, restaurant_reviews, repeat, todo_list, flight_status, last_maintenance, pto_balance, directions, restaurant_reservation, confirm_reservation, order_status, tell_joke, oos, calculator, gas_type, travel_suggestion, share_location, shopping_list, change_language, timezone, yes, book_flight, payday, no, meeting_schedule, do_you_have_pets, order, credit_limit_change, recipe, calendar_update, improve_credit_score, pin_change, update_playlist, cancel_reservation, date, measurement_conversion, definition, oil_change_how, whisper_mode, flip_coin, find_phone, todo_list_update, redeem_rewards, change_user_name, change_ai_name, min_payment, spelling, how_busy, change_accent, schedule_meeting, food_last, international_fees, book_hotel, where_are_you_from, change_volume, are_you_a_bot, tire_change, ingredients_list, current_location, goodbye, insurance_change, w2, pto_used, new_card, traffic, balance, fun_fact, ingredient_substitution, transfer, freeze_account, schedule_maintenance, thank_you, what_is_your_name, gas, vaccines, accept_reservations, transactions, cook_time, distance, routing, report_lost_card, next_song, carry_on, what_song, travel_notification, apr, nutrition_info, who_made_you, next_holiday, bill_balance, reminder_update, how_old_are_you, mpg, rollover_401k, account_blocked, plug_type, card_declined, smart_home, meal_suggestion, direct_deposit, sync_device, pay_bill, uber, pto_request_status, taxes, order_checks, greeting, income, spending_history, calories, reset_settings, expiration_date, tire_pressure, meaning_of_life, make_call, insurance, restaurant_suggestion, interest_rate, credit_limit, bill_due, who_do_you_work_for, report_fraud, roll_dice, change_speed, damaged_card, shopping_list_update, what_are_your_hobbies, lost_luggage, oil_change_when, text, travel_alert, car_rental, international_visa, application_status, jump_start, credit_score, what_can_i_ask_you, replacement_card_duration, pto_request, national_id, company_name, account_number, password, url, -47, -4, 14, 34, 24, 5760, 5756, 5750, 5765, 33, -2, 8, 12, 29, 16, 52, -5, 17, 72, 80, 78, 103, 98, 48, 200, 150, 180, 196, 64, 114, 59, 84, 38, 63, 43, 32, 21, 20, -18, 27, 149, 197, 172, 147, 88, 93, -1, -3, 350, 397, 400, 375, 248, 254, 255, 251, 6580, 6598, 6600, 6593, 293, 305, 288, 13997, 14000, 13993, 14007, -33, 475, 504, 501, 500, 11, 40, 90, 70, 89, 1198, 1194, 1202, 1173, 297, 310, -15, -10, 71, 121, -21, 36, 122, 128, 125, 190, 207, 47, 39, 92, 130, 76, 195, 41, 46, 283, 273, 44, 18, -8, 62, 5, 55, play game, iot coffee, transport traffic, calendar query, qa factoid, play audiobook, iot cleaning, alarm remove, general negate, recommendation events, alarm set, play podcasts, transport query, general commandstop, iot wemo off, music settings, general explain, lists query, general joke, iot hue lightup, email query, takeaway query, play radio, lists remove, music query, recommendation movies, close_no_action, delivery, stop, harmful, qa_stock, recommendation_movies, audio_volume_up, play_audiobook, recommendation_locations, cooking_recipe, datetime_convert, cooking_query, qa_currency, weather_query, music_query, iot_hue_lightdim, iot_hue_lightup, qa_definition, audio_volume_down, general_greet, iot_hue_lightoff, alarm_set, general_quirky, play_podcasts, email_addcontact, cooking, IN:LIKE_MUSIC, IN:GET_RECIPES, IN:GET_TIMER, IN:CREATE_PLAYLIST_MUSIC, IN:PAUSE_MUSIC, IN:SET_DEFAULT_PROVIDER_CALLING, IN:SWITCH_CALL, IN:SET_UNAVAILABLE, IN:GET_EDUCATION_DEGREE, IN:GET_MAJOR, IN:GET_TRACK_INFO_MUSIC, IN:GET_ATTENDEE_EVENT, IN:UPDATE_METHOD_CALL, IN:GET_AVAILABILITY, IN:GET_EMPLOYMENT_TIME, IN:GET_REMINDER_DATE_TIME, IN:CREATE_REMINDER, IN:SKIP_TRACK_MUSIC, IN:GET_AGE, IN:GET_LIFE_EVENT, IN:STOP_SHUFFLE_MUSIC, IN:GET_CALL, IN:MERGE_CALL, IN:DELETE_ALARM, IN:UNLOOP_MUSIC, IN:SET_DEFAULT_PROVIDER_MUSIC, IN:RESUME_CALL, IN:CREATE_CALL, IN:GET_CALL_TIME, IN:GET_CONTACT_METHOD, IN:GET_UNDERGRAD, IN:GET_GENDER, IN:SHARE_EVENT, IN:RESTART_TIMER, IN:SET_RSVP_INTERESTED, IN:QUESTION_NEWS, IN:GET_DETAILS_NEWS, IN:ADD_TIME_TIMER, IN:GET_MESSAGE_CONTACT, IN:ANSWER_CALL, IN:SET_AVAILABLE, IN:UPDATE_REMINDER_LOCATION, IN:REMOVE_FROM_PLAYLIST_MUSIC, IN:PLAY_MEDIA, IN:HOLD_CALL, IN:GET_STORIES_NEWS, IN:GET_EVENT, IN:FOLLOW_MUSIC, IN:GET_AIRQUALITY, IN:CANCEL_CALL, IN:GET_REMINDER_AMOUNT, IN:GET_WEATHER, IN:UPDATE_REMINDER_DATE_TIME, IN:CANCEL_MESSAGE, IN:IS_TRUE_RECIPES, IN:GET_GROUP, IN:PAUSE_TIMER, IN:DISPREFER, IN:CREATE_TIMER, option_9, option_10, option_11, option_12, option_13, option_14, option_15, mixed, SearchHotel, MakePayment, RequestPayment, GetTimesForMovie, BuyMovieTickets, FindRestaurants, SearchHouse, SearchOnewayFlight, LookupMusic, AddAlarm, GetWeather, GetRide, ShareLocation, teacher, supervisor, clerk, lawyer, baker, carpenter, developer, auditor

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 4329 | 0.2296 | 0.0573 | 0.0437 | 0.7721 | 1.8430 | 0.6643 | 0.3202 | 0.2060 | 0.78/0.78 | 0.1388 | 0.373 ms | 1.819 ms (44) | ✓ | s 0.0100 / d 0.5756 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.2060 · calibrated 0.2060 · conformal-naive floor 0.4061 → **FAIL** (does not beat both)

## s1mb_noul — 6173 cases / 6173 questions

**slice-integrity: test 6173 (fnv1a64-84180b47f6a27e92) · cal 200 (fnv1a64-8ef8b387238526af) · pool 5974 (fnv1a64-bae132b5e4d2771b) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.5000 |
| 1 | observed-laplace | 0 | bag | 0.5150 |
| 4 | observed-laplace | 0 | bag | 0.5150 |
| 16 | observed-laplace | 0 | bag | 0.5150 |
| 32 | observed-laplace | 0 | bag | 0.5150 |
| 64 | observed-laplace | 0 | bag | 0.5150 |
| 1 | fixed-1 | 0 | bag | 0.5300 |
| 4 | fixed-1 | 0 | bag | 0.5300 |
| 16 | fixed-1 | 0 | bag | 0.5300 |
| 32 | fixed-1 | 0 | bag | 0.5300 |
| 64 | fixed-1 | 0 | bag | 0.5300 |
| 1 | observed-laplace | 1 | bag | 0.4850 |
| 4 | observed-laplace | 1 | bag | 0.4850 |
| 16 | observed-laplace | 1 | bag | 0.4850 |
| 32 | observed-laplace | 1 | bag | 0.4850 |
| 64 | observed-laplace | 1 | bag | 0.4850 |
| 1 | fixed-1 | 1 | bag | 0.4700 |
| 4 | fixed-1 | 1 | bag | 0.4700 |
| 16 | fixed-1 | 1 | bag | 0.4700 |
| 32 | fixed-1 | 1 | bag | 0.4700 |
| 64 | fixed-1 | 1 | bag | 0.4700 |

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1200 · max_prob 0.0016 · inv_entropy 0.1200

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 6173 | 0.7055 | 0.4291 | 0.2042 | 0.5001 | 0.6933 | 0.3751 | 0.6970 | 0.0061 | 1.00/0.47 | 0.3040 | 0.291 ms | 0.431 ms (62) | ✓ | s 0.4550 / d 0.6012 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.1098 · calibrated 0.0061 · conformal-naive floor 0.0614 → **PASS** (beats both the uncalibrated output AND the floor)

## s1mb_score — 2571 cases / 2571 questions

**slice-integrity: test 2574 (fnv1a64-49574479b2c3dc4d) · cal 200 (fnv1a64-f36037b81b6b2013) · pool 2374 (fnv1a64-345fee4f7512445c) · cal missing 5: 5,6,7,8,9 · pool missing 5: 5,6,7,8,9 · OK**

**corpus cap:** 16 (registry)

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.2172 |
| 1 | observed-laplace | — | bag | 0.2172 |
| 4 | observed-laplace | — | bag | 0.2172 |
| 16 | observed-laplace | — | bag | 0.2172 |
| 32 | observed-laplace | — | bag | 0.2172 |
| 64 | observed-laplace | — | bag | 0.2172 |
| 1 | fixed-1 | — | bag | 0.2172 |
| 4 | fixed-1 | — | bag | 0.2172 |
| 16 | fixed-1 | — | bag | 0.2172 |
| 32 | fixed-1 | — | bag | 0.2172 |
| 64 | fixed-1 | — | bag | 0.2172 |

⛔ **corpus fallback (Issue 039 guard):** 5 option label(s) with NO train docs in the corpus pool — self-doc fallback only: 5, 6, 7, 8, 9

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2571 | 0.4982 | 0.1016 | 0.2923 | 0.7941 | 1.5852 | 0.5904 | 0.4350 | 0.2681 | 1.00/0.08 | 0.5129 | 0.245 ms | 0.379 ms (26) | ✓ | s 0.2301 / d 0.4153 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.0041 · calibrated 0.2681 · conformal-naive floor 0.4870 → **FAIL** (does not beat both)

## typed_decisions — 400 cases / 2000 questions

**slice-integrity: test 400 (fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100 (fnv1a64-a1679de56e9278b1) · OK**

**corpus cap:** 48 (registry)

**count tables:** cal-selected scale 0 α off view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 ← selected | off | — | bag | 0.3180 |
| 1 | observed-laplace | — | bag | 0.3180 |
| 4 | observed-laplace | — | bag | 0.3180 |
| 16 | observed-laplace | — | bag | 0.3180 |
| 32 | observed-laplace | — | bag | 0.3180 |
| 64 | observed-laplace | — | bag | 0.3180 |
| 1 | fixed-1 | — | bag | 0.3180 |
| 4 | fixed-1 | — | bag | 0.3180 |
| 16 | fixed-1 | — | bag | 0.3180 |
| 32 | fixed-1 | — | bag | 0.3180 |
| 64 | fixed-1 | — | bag | 0.3180 |

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.5529 · max_prob 0.2031 · inv_entropy 0.5529

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.5700 | 0.5393 | 0.2154 | 0.6434 | 1.1467 | 0.3059 | 0.6680 | 0.0946 | 1.00/0.63 | 0.5747 | 0.762 ms | 1.623 ms (5) | ✓ | s 0.5650 / d 0.9122 (n 100) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 2000 | 0.5700 | 0.5393 | 0.2925 | 0.3922 | ✓ |


**G1 (modelless readout ECE):** raw 0.5645 · calibrated 0.0946 · conformal-naive floor 0.2164 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3330 · brier_soft 0.2107 · score MAE 0.6698 · within_1 0.7600

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.5483 | 0.2756 | 0.2727 |
| modelless | noul | 600 | 0.7433 | 0.2013 | 0.5420 |
| modelless | score | 800 | 0.4562 | 0.1809 | 0.2754 |

## ag_news — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.3900 |
| 1 | observed-laplace | — | bag | 0.7000 |
| 4 ← selected | observed-laplace | — | bag | 0.7450 |
| 16 | observed-laplace | — | bag | 0.7450 |
| 32 | observed-laplace | — | bag | 0.7450 |
| 64 | observed-laplace | — | bag | 0.7450 |
| 1 | fixed-1 | — | bag | 0.7000 |
| 4 | fixed-1 | — | bag | 0.7450 |
| 16 | fixed-1 | — | bag | 0.7450 |
| 32 | fixed-1 | — | bag | 0.7450 |
| 64 | fixed-1 | — | bag | 0.7450 |

**transductive column (NOT the headline):** acc 0.8825 vs honest 0.8825 (+0.0 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.8204 · max_prob 0.4857 · inv_entropy 0.8204

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8825 | 0.8742 | 0.4783 | 0.5053 | 0.9630 | 0.0469 | 0.9400 | 0.0459 | 1.00/0.49 | 0.9510 | 0.143 ms | 0.240 ms (5) | ✓ | s 0.8359 / d 0.4105 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.8825 | 0.8742 | 0.3075 | 0.8303 | ✓ |


**G1 (modelless readout ECE):** raw 0.8305 · calibrated 0.0459 · conformal-naive floor 0.2482 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15799 (fnv1a64-8ef2b2af400721fd) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.1950 |
| 1 | observed-laplace | — | bag | 0.5350 |
| 4 ← selected | observed-laplace | — | bag | 0.5500 |
| 16 | observed-laplace | — | bag | 0.5500 |
| 32 | observed-laplace | — | bag | 0.5500 |
| 64 | observed-laplace | — | bag | 0.5500 |
| 1 | fixed-1 | — | bag | 0.4650 |
| 4 | fixed-1 | — | bag | 0.4850 |
| 16 | fixed-1 | — | bag | 0.4850 |
| 32 | fixed-1 | — | bag | 0.4850 |
| 64 | fixed-1 | — | bag | 0.4850 |

**transductive column (NOT the headline):** acc 0.8875 vs honest 0.8850 (+0.2 pt; 400 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.7589 · max_prob 0.5274 · inv_entropy 0.7589

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.8850 | 0.8014 | 0.6182 | 0.6593 | 1.3643 | 0.0360 | 0.9750 | 0.0488 | 1.00/0.46 | 0.9349 | 0.110 ms | 0.121 ms (5) | ✓ | s 0.7631 / d 0.4265 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.8850 | 0.8014 | 0.2975 | 0.8363 | ✓ |


**G1 (modelless readout ECE):** raw 0.8621 · calibrated 0.0488 · conformal-naive floor 0.2781 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**slice-integrity: test 600 (fnv1a64-67fe9666fa0c3a6f) · cal 200 (fnv1a64-d210e7c6707c3d40) · pool 8333 (fnv1a64-b46434b82644c53b) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 16 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.2400 |
| 1 | observed-laplace | — | bag | 0.3300 |
| 4 | observed-laplace | — | bag | 0.3550 |
| 16 ← selected | observed-laplace | — | bag | 0.3600 |
| 32 | observed-laplace | — | bag | 0.3600 |
| 64 | observed-laplace | — | bag | 0.3600 |
| 1 | fixed-1 | — | bag | 0.3150 |
| 4 | fixed-1 | — | bag | 0.3400 |
| 16 | fixed-1 | — | bag | 0.3400 |
| 32 | fixed-1 | — | bag | 0.3350 |
| 64 | fixed-1 | — | bag | 0.3350 |

**transductive column (NOT the headline):** acc 0.3967 vs honest 0.3967 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2755 · max_prob 0.0465 · inv_entropy 0.2755

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.3967 | 0.3210 | 0.1389 | 0.7604 | 1.5156 | 0.5365 | 0.4700 | 0.1086 | 1.00/0.53 | 0.4057 | 0.090 ms | 0.094 ms (7) | ✓ | s 0.2813 / d 0.4166 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 600 | 0.3967 | 0.3210 | 0.2733 | 0.1697 | ✓ |


**G1 (modelless readout ECE):** raw 0.3849 · calibrated 0.1086 · conformal-naive floor 0.2220 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1170 · within_1 0.5400

## prompt_injections — 116 cases / 116 questions

**slice-integrity: test 116 (fnv1a64-a6a375f0273e0bef) · cal 100 (fnv1a64-a45e6561b2a98ff1) · pool 446 (fnv1a64-8b32b961bbd47195) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.5000 |
| 1 | observed-laplace | 0 | bag | 0.3900 |
| 4 | observed-laplace | 0 | bag | 0.3900 |
| 16 | observed-laplace | 0 | bag | 0.3900 |
| 32 | observed-laplace | 0 | bag | 0.3900 |
| 64 | observed-laplace | 0 | bag | 0.3900 |
| 1 | fixed-1 | 0 | bag | 0.5000 |
| 4 | fixed-1 | 0 | bag | 0.4900 |
| 16 | fixed-1 | 0 | bag | 0.4900 |
| 32 | fixed-1 | 0 | bag | 0.4900 |
| 64 | fixed-1 | 0 | bag | 0.4900 |
| 1 ← selected | observed-laplace | 1 | bag | 0.6100 |
| 4 | observed-laplace | 1 | bag | 0.6100 |
| 16 | observed-laplace | 1 | bag | 0.6100 |
| 32 | observed-laplace | 1 | bag | 0.6100 |
| 64 | observed-laplace | 1 | bag | 0.6100 |
| 1 | fixed-1 | 1 | bag | 0.5100 |
| 4 | fixed-1 | 1 | bag | 0.5100 |
| 16 | fixed-1 | 1 | bag | 0.5100 |
| 32 | fixed-1 | 1 | bag | 0.5100 |
| 64 | fixed-1 | 1 | bag | 0.5100 |

**transductive column (NOT the headline):** acc 0.7672 vs honest 0.7672 (+0.0 pt; 0 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.6831 · max_prob 0.1165 · inv_entropy 0.6831

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.7672 | 0.7643 | 0.1336 | 0.3365 | 0.5194 | 0.0949 | 0.8966 | 0.0949 | 1.00/0.68 | 0.9459 | 0.076 ms | 0.085 ms (2) | ✓ | s 0.7991 / d 0.4619 (n 100) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 116 | 0.7672 | 0.7643 | 0.5172 | 0.5179 | ✓ |


**G1 (modelless readout ECE):** raw 0.6745 · calibrated 0.0949 · conformal-naive floor 0.3622 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK**

**corpus cap:** 64 (registry)

**count tables:** cal-selected scale 4 α fixed-1 view pair (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.3650 |
| 1 | observed-laplace | — | bag | 0.3100 |
| 4 | observed-laplace | — | bag | 0.3400 |
| 16 | observed-laplace | — | bag | 0.3250 |
| 32 | observed-laplace | — | bag | 0.3250 |
| 64 | observed-laplace | — | bag | 0.3200 |
| 1 | fixed-1 | — | bag | 0.3100 |
| 4 | fixed-1 | — | bag | 0.3300 |
| 16 | fixed-1 | — | bag | 0.3250 |
| 32 | fixed-1 | — | bag | 0.3300 |
| 64 | fixed-1 | — | bag | 0.3300 |
| 1 | observed-laplace | — | pair | 0.5600 |
| 4 | observed-laplace | — | pair | 0.5500 |
| 16 | observed-laplace | — | pair | 0.5500 |
| 32 | observed-laplace | — | pair | 0.5500 |
| 64 | observed-laplace | — | pair | 0.5500 |
| 1 | fixed-1 | — | pair | 0.5600 |
| 4 ← selected | fixed-1 | — | pair | 0.5800 |
| 16 | fixed-1 | — | pair | 0.5700 |
| 32 | fixed-1 | — | pair | 0.5650 |
| 64 | fixed-1 | — | pair | 0.5550 |

**transductive column (NOT the headline):** acc 0.5033 vs honest 0.5233 (-2.0 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.5868 · max_prob 0.2135 · inv_entropy 0.5868

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.5233 | 0.5061 | 0.1383 | 0.6178 | 1.0287 | 0.3120 | 0.6400 | 0.1119 | 1.00/0.54 | 0.6087 | 0.084 ms | 0.088 ms (4) | ✓ | s 0.5569 / d 0.5110 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.5233 | 0.5061 | 0.3333 | 0.2850 | ✓ |


**G1 (modelless readout ECE):** raw 0.5133 · calibrated 0.1119 · conformal-naive floor 0.1480 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK**

**corpus cap:** 48 (registry)

**count tables:** cal-selected scale 4 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4450 |
| 1 | observed-laplace | — | bag | 0.5800 |
| 4 ← selected | observed-laplace | — | bag | 0.6000 |
| 16 | observed-laplace | — | bag | 0.5900 |
| 32 | observed-laplace | — | bag | 0.5900 |
| 64 | observed-laplace | — | bag | 0.5900 |
| 1 | fixed-1 | — | bag | 0.5450 |
| 4 | fixed-1 | — | bag | 0.5800 |
| 16 | fixed-1 | — | bag | 0.5600 |
| 32 | fixed-1 | — | bag | 0.5600 |
| 64 | fixed-1 | — | bag | 0.5600 |

**transductive column (NOT the headline):** acc 0.7833 vs honest 0.7800 (+0.3 pt; 300 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.7800 | 0.7698 | 0.6352 | 0.7856 | 2.0599 | 0.0553 | 0.9667 | 0.0790 | 0.51/0.31 | 0.7681 | 0.111 ms | 0.133 ms (4) | ✓ | s 0.1106 / d 0.5259 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.7800 | 0.7698 | 0.0867 | 0.7591 | ✓ |


**G1 (modelless readout ECE):** raw 0.6352 · calibrated 0.0790 · conformal-naive floor 0.1163 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK**

**corpus cap:** 40 (registry)

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.4650 |
| 1 ← selected | observed-laplace | — | bag | 0.8100 |
| 4 | observed-laplace | — | bag | 0.8050 |
| 16 | observed-laplace | — | bag | 0.8050 |
| 32 | observed-laplace | — | bag | 0.8050 |
| 64 | observed-laplace | — | bag | 0.8000 |
| 1 | fixed-1 | — | bag | 0.7950 |
| 4 | fixed-1 | — | bag | 0.7750 |
| 16 | fixed-1 | — | bag | 0.7700 |
| 32 | fixed-1 | — | bag | 0.7600 |
| 64 | fixed-1 | — | bag | 0.7600 |

**transductive column (NOT the headline):** acc 0.8360 vs honest 0.8420 (-0.6 pt; 500 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.8420 | 0.8362 | 0.8194 | 0.9688 | 3.8215 | 0.0599 | 0.9720 | 0.0517 | 1.00/0.48 | 0.9349 | 0.357 ms | 0.666 ms (6) | ✓ | s 0.7478 / d 0.5549 (n 200) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 500 | 0.8420 | 0.8362 | 0.0140 | 0.8398 | ✓ |


**G1 (modelless readout ECE):** raw 0.8194 · calibrated 0.0517 · conformal-naive floor 0.2933 → **PASS** (beats both the uncalibrated output AND the floor)

## code_fixtures — 16 cases / 32 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4602 · max_prob 0.1479 · inv_entropy 0.4602

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 32 | 0.3750 | 0.1640 | 0.0948 | 0.6858 | 1.3870 | 0.4073 | 0.6250 | 0.0938 | 1.00/0.34 | 0.2857 | 0.215 ms | 0.372 ms (1, max@case9) | ✓ | s 0.4607 / d 0.3365 (n 64) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 32 | 0.3750 | 0.1640 | 0.3750 | 0.0000 | ✓ |


**G1 (modelless readout ECE):** raw 0.3742 · calibrated 0.0938 · conformal-naive floor 0.3510 → **PASS** (beats both the uncalibrated output AND the floor)

## semantic_defects — 102 cases / 102 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

**count tables:** cal-selected scale 1 α observed-laplace view bag (promotion bar +5 pt over off)

| nb scale | α | noul yes→domain | view | cal acc |
|---|---|---|---|---|
| 0 | off | — | bag | 0.2778 |
| 1 ← selected | observed-laplace | — | bag | 0.4444 |
| 4 | observed-laplace | — | bag | 0.4444 |
| 16 | observed-laplace | — | bag | 0.4444 |
| 32 | observed-laplace | — | bag | 0.4444 |
| 64 | observed-laplace | — | bag | 0.4444 |
| 1 | fixed-1 | — | bag | 0.3333 |
| 4 | fixed-1 | — | bag | 0.3889 |
| 16 | fixed-1 | — | bag | 0.3889 |
| 32 | fixed-1 | — | bag | 0.3889 |
| 64 | fixed-1 | — | bag | 0.3889 |

**transductive column (NOT the headline):** acc 0.4314 vs honest 0.4412 (-1.0 pt; 102 pseudo-labelled test docs) — TRANSDUCTIVE — unlabeled TEST text joins the count tables, labelled by the honest engine's own forced picks (gold never read); 2-fold cross-fit (half A's pseudo-docs re-score half B and vice versa, so no case scores against its own text); drafter corpora, route, heads and posture unchanged. Not comparable to the headline acc

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 102 | 0.4412 | 0.4335 | 0.2347 | 0.7987 | 1.6978 | 0.4301 | 0.4902 | 0.4358 | 0.50/0.50 | 0.4314 | 0.024 ms | 0.026 ms (2) | ✓ | s 0.0027 / d 0.5209 (n 18) |
| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |

**G1 (modelless readout ECE):** raw 0.4358 · calibrated 0.4358 · conformal-naive floor 0.1945 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## Landscape — published specialist rows (NOT measured by this harness)

External "System One" typed-decision models on the same 400-case /
2000-question Typed Decisions official test split, quoted AS PUBLISHED
(issue 025; pin `malevrigns/agent-jev` @ `a965ca8f`, Apache-2.0). Their
protocol differs from ours — the footnotes are part of the row; no number
here is comparable without them.

| lane · model | source | acc | bool·noul / choice / score | p50 case |
|---|---|---|---|---|
| AgentJev-0.6B (598M, Qwen3-0.6B backbone) | published (their run) | **0.7925** | 88.83 / 75.33 / 75.00 | ~60–70 ms, their cuda box |
| reflex · agentjev (their service, gold-label) | **MEASURED** (bench 039, 4090) | **0.7715** | — | 88 ms (loopback HTTP) |
| Laya (published checkpoint, 421M ModernBERT) | their table — card-copied, not re-scored | 0.7700 | — | 41.53 ms (their box) |
| TypeSafe Jev 1.13.0 | their table — zero-shot generalist | 0.727 | — | — |
| reflex · laya-riir·typed | MEASURED — the typed_decisions table above | 0.7445 (baseline `aa37823`) | 78.50 / 73.33 / 72.25 | 1312 ms (m3 metal, that baseline) |
| reflex · modelless | MEASURED — the typed_decisions table above | 0.3190 (baseline `aa37823`) | 53.17 / 18.67 / 25.87 | 0.472 ms |

Footnotes: (1) their accuracy is agreement with the public TEACHER argmax;
ours is gold-label under the standard harness protocol — different
references of truth. (1b) the MEASURED agentjev row (bench 039, Issue 025
amendment 4) closes that gap for AgentJev: **0.7715 gold-label** on this
harness's split (teacher-argmax 0.7925 → gold −2.1pt) — the published
ranking SURVIVES the protocol change (+2.7pt over our measured laya-typed
0.7445; their teacher-protocol gap was +2.25). (2) their run held out 120 dev + 120 cal cases and
selected the step-600 checkpoint on dev soft-CE before opening test; ours
fits no per-benchmark head. (3) their wide-load figure (shared-prefix
298.91 ms vs unshared 609.65 ms at 66 paths / 33,547 tokens, backbone
token-ops 33,547 → 2,551 = 92.4% reduction, max prob delta 5.08e-4) is
their box and their load — not re-measured here. (4) on SHORT inputs their
own table reads Laya faster (41.53 ms vs ~60–70 ms p50/case); AgentJev's
latency win is wide candidate loads only. (5) the bool·noul column maps
their boolean primitive to our noul primitive — the closest analogue, not
a wire match; neither of their lanes carries an abstention primitive (the
wire's first-class abstention is ours alone).

## Landscape — fast-decisions (vendor suite; NOT measured by this harness)

fastino's `fast-decisions` suite — 17 English operational-decision
domains (commerce support intent/topic, ticket routing, product feedback,
banking intent, document type, review sentiment, assistant handoff,
email triage, clinic request, travel request, news topic, paper field,
sports recap, restaurant review, benefits request, screen tags), 300
held-out test examples per domain, exact-match accuracy, the same text
and candidate labels for every model. Quoted AS PUBLISHED (issue 029).

| model | avg exact-match |
|---|---|
| GLiNER2.5-Decide (340M, DeBERTa-v3-large) | **60.2%** |
| GLiNER2.5-Decide-1B (their dataset card: "GLiNER2 XL (1B)") | 59.6% |
| JevK5 | 57.6% |
| GLiNER2.5-multi-Decide (287M) | 56.7% |
| SemIf (Qwen3.5-4B) | 56.4% |
| GLiFormer large-v1 | 49.0% |
| Laya Router | 46.6% |

Footnotes: (1) the scored split is private — their public repo carries
only the 100/domain development split with an explicit "do not report a
score computed on the files in this repo" note, so no row here can be
re-scored outside fastino. (2) their metric is per-head exact match
(single-label string equality; multi-label heads compared as sets),
averaged over the 17 domains. (3) their "Laya Router" row names no
checkpoint variant — neither the base router nor the typed specialist;
our measured laya-riir cells are the port checkpoints under OUR protocol,
so the 60.2-vs-46.6 gap is their-suite/their-checkpoint/their-protocol.
(4) our measured comparison lives in the 15-suite tables (bench 037,
`.benchmarks/037_gliner_lane_4090`): the DIRECTION is confirmed on
decision-style suites — gliner beats the laya base checkpoint 9/15
(banking77 +20.8pt, massive_intent +7.3pt, all five harness families) —
and honestly refuted on classic NLU (ag_news −24.8pt, xnli_en −38.3pt;
their card's own "not a general-purpose model" framing). The
typed_decisions headline stays laya's: the `typed` specialist 0.7445 vs
gliner 0.5280.

